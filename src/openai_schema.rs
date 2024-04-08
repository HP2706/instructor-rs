use schemars::JsonSchema;
use validator::ValidateArgs;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::any::type_name;
use crate::error::Error;
use crate::enums::InstructorResponse;
use crate::enums::IterableOrSingle;
use crate::mode::Mode;
use crate::utils::extract_json_from_codeblock;
use async_openai::types::CreateChatCompletionResponse;
use async_openai::types::{ChatCompletionMessageToolCall, FunctionObject };

pub trait BaseSchema: 
     Debug + Serialize + for<'de> Deserialize<'de> + 
    ValidateArgs<'static> + JsonSchema + Sized + Send + Sync + Clone + 'static {}

impl<T> BaseSchema for T
where T: 
    Debug + Serialize + for<'de> Deserialize<'de> + 
    ValidateArgs<'static> + JsonSchema + Sized + Send + Sync + Clone + 'static {}

pub trait BaseArg: 
    Clone + Send + Sync +'static {}

impl<A> BaseArg for A
where A: Clone + Send + Sync  + 'static{}


///this is the trait that implements the functionality similar to OpenAISchema in the instructor python library
/// in order to use with your struct, your strutc must implement the following traits:
/// JsonSchema, Serialize, Debug, Default, Validate, Deserialize, Clone
/// this can be done either via #[derive(JsonSchema, Serialize, Debug, Default, Validate, Deserialize, Clone)] or by importing
/// model_traits_macro::derive_all and calling [derive_all] on your struct
/// 
/// Example
/// 
/// #[derive_all]
/// struct MyStruct {
///     a: i32,
///     b: i32,
/// }
/// 
/// now you can access the following methods:
/// 
/// Mystruct::openai_schema(...)
/// Mystruct::tool_schema(...)
/// Mystruct::model_validate_json(...)
/// Mystruct::from_response(...)
pub trait OpenAISchema<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + BaseSchema,
    Args: BaseArg,
{
    type Args : BaseArg;

    ///returns the openai schema for the struct in a nice format that the LLM can understand
    fn openai_schema() -> String; 

    ///returns the openai schema for the struct as a FunctionObject 
    /// that can be used in tools field (functions are deperecated)
    fn tool_schema() -> FunctionObject;
 
    ///parses the model from string to struct and does struct validation
    /// # Arguments
    /// 
    /// * `model` - The model to use (IterableOrSingle::Iterable(model) or IterableOrSingle::Single(model)) 
    ///     if Iterable() is used the model will know to parse an array of json ie {...},{...} into two structs 
    ///     if Single() is used the model will know to parse a single json ie {...} into a single struct
    /// * `data` - The string to parse
    /// * `validation_context` - The validation context to use 
    /// (this is if you have validator functions that require custom context exactly like validation_context in pydantic)
    /// # Returns
    /// * `InstructorResponse::One(data)` - If the model is a single object
    /// * `InstructorResponse::Many(data)` - If the model is an iterable of objects
    fn model_validate_json(
        model: &IterableOrSingle<Self>, 
        data: &str, 
        validation_context: &Args
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;
    
    ///takes a response and parses it into the struct using functions like model_validate_json()
    /// #Arguments
    /// * `model`  - The model to use (IterableOrSingle::Iterable(model) or IterableOrSingle::Single(model)) 
    ///     if Iterable() is used the model will know to parse an array of json ie {...},{...} into two structs 
    ///     if Single() is used the model will know to parse a single json ie {...} into a single struct
    /// * `response` - The response to parse
    /// * `validation_context` - The validation context to use 
    ///     (this is if you have validator functions that require custom context exactly like validation_context in pydantic)
    /// * `mode` - The mode to extract the json in 
    /// # Returns
    /// * `InstructorResponse::One(data)` - If the model is a single object
    /// * `InstructorResponse::Many(data)` - If the model is an iterable of objects
    fn from_response(
        model: &IterableOrSingle<Self>,
        response: &CreateChatCompletionResponse,
        validation_context: &Args,
        mode: Mode,
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;
    
    ///this function
    fn parse_json(
        model: &IterableOrSingle<Self>,
        completion: &CreateChatCompletionResponse,
        validation_context: &Args,
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;

    fn parse_tools(
        model: &IterableOrSingle<Self>,
        completion: &CreateChatCompletionResponse,
        validation_context: &Args,
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;

}

impl<A, T> OpenAISchema<A, T> for T
where
    T: ValidateArgs<'static, Args=A> + BaseSchema,
    A: BaseArg,
{
    type Args = A;

    // The rest of your implementation remains the same..

    fn openai_schema() -> String 
    where 
        T: serde::Serialize + JsonSchema 
    {
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_value(&schema).unwrap();
        serde_json::to_string_pretty(&schema_json).unwrap()
    }

    fn tool_schema() -> FunctionObject {
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_value(&schema).unwrap();
        let title = schema_json["title"].as_str().unwrap_or_default().to_string();
    
        let description = match schema_json["description"].as_str() {
            Some(desc) => desc.to_string(),
            None => format!("Correctly extracted `{}` with all the required parameters with correct types", title),
        };
    
        let mut parameters = serde_json::Map::new();
    
        if let Some(props) = schema_json["properties"].as_object() {
            for (prop_name, prop_value) in props {
                let mut prop_schema = prop_value.clone();
    
                if let Some(all_of) = prop_value["allOf"].as_array() {
                    if let Some(ref_value) = all_of[0]["$ref"].as_str() {
                        if let Some(def_name) = ref_value.split('/').last() {
                            if let Some(def_value) = schema_json["definitions"][def_name].as_object() {
                                if let Some(enum_values) = def_value["enum"].as_array() {
                                    prop_schema["enum"] = serde_json::Value::Array(enum_values.clone());
                                }
                            }
                        }
                    }
                }
    
                parameters.insert(prop_name.to_string(), prop_schema);
            }
        }
    
        let required = schema_json["required"].as_array().unwrap_or(&Vec::new()).iter()
            .map(|r| r.as_str().unwrap().to_string())
            .collect::<Vec<String>>();
    
        FunctionObject {
            name: title,
            description: Some(description),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": parameters,
                "required": required
            })),
        }
    }

    fn model_validate_json(
        model: &IterableOrSingle<Self>, 
        data: &str, 
        validation_context: &Self::Args
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema,
    {

        match model {
            IterableOrSingle::Iterable(_) => {
                let bracketed_data = &format!("[{}]", data);
                let data = serde_json::from_str::<Vec<T>>(bracketed_data)
                    .map_err(|e| Error::SerdeError(e)); // Convert serde_json::Error to custom Error::SerdeError
                data.and_then(|data| {
                    data.into_iter().map(|item| validate_single(item, validation_context.clone()))
                        .collect::<Result<Vec<T>, Error>>() 
                        .map(|data| InstructorResponse::Many(data)) 
                })
            },
            IterableOrSingle::Single(_) => {
                let data = serde_json::from_str::<T>(data);
                match data {
                    Ok(data) => {
                        let validated_data = validate_single(data, validation_context.clone()); 
                        validated_data.map(|data| InstructorResponse::One(data)) 
                    },
                    Err(e) => Err(Error::SerdeError(e)),
                }
            }
        }
    }

    fn from_response(
        model: &IterableOrSingle<Self>,
        response: &CreateChatCompletionResponse,
        validation_context: &Self::Args,
        mode: Mode,
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema,
    {
        match mode {
            Mode::JSON | Mode::JSON_SCHEMA | Mode::MD_JSON => {
                Self::parse_json(model, response, validation_context)
            }
            Mode::TOOLS => {
                println!("\n\nMode::TOOLS response: {:?}", response);
                return Self::parse_tools(model, response, validation_context);
            }
            _ => Err(Error::NotImplementedError(
                "This feature is not yet implemented.".to_string(),
            )),
        }
    }

    ///this function is used to parse a string to multiple json objects, however the complexity of parsing is placed in model_validate_json
    /// #Arguments
    /// * `model` - The model to use (IterableOrSingle::Iterable(model) or IterableOrSingle::Single(model)) 
    ///     if Iterable() is used the model will know to parse an array of json ie {...},{...} into two structs 
    ///     if Single() is used the model will know to parse a single json ie {...} into a single struct
    /// * `completion` - The response to parse
    /// * `validation_context` - The validation context to use 
    ///     (this is if you have validator functions that require custom context exactly like validation_context in pydantic)
    fn parse_json(
        model: &IterableOrSingle<Self>,
        completion: &CreateChatCompletionResponse,
        validation_context: &Self::Args,
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema,
    {
        let text = completion.choices[0].message.content.clone().unwrap();
        println!("text: {}", text);
        let json_extract = extract_json_from_codeblock(&text);
        match json_extract {
            Ok(json_extract) => {
                return Self::model_validate_json(model, &json_extract, validation_context);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    ///this function is used to parse the tools field in the response to one or more structs of type Self
    /// # Arguments:
    /// * `model` - The model to use (IterableOrSingle::Iterable(model) or IterableOrSingle::Single(model)) 
    ///     if Iterable() is used the model will know to parse an array of json ie {...},{...} into two structs 
    ///     if Single() is used the model will know to parse a single json ie {...} into a single struct
    /// * `completion` - The response to parse
    /// * `validation_context` - The validation context to use 
    ///     (this is if you have validator functions that require custom context exactly like validation_context in pydantic)
    fn parse_tools(
        model: &IterableOrSingle<Self>,
        completion: &CreateChatCompletionResponse,
        validation_context: &Self::Args,
    ) -> Result<InstructorResponse<T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema,
    {
        let message = &completion.choices[0].message;
        match model {
            IterableOrSingle::Single(_) => {
                match &message.tool_calls {
                    Some(tool_calls) => {
                        if tool_calls.len() != 1 {
                            return Err(Error::Generic("Expected exactly one tool call".to_string()));
                        }
                        let tool_call = &tool_calls[0];
                        let out = check_tool_call::<T>(tool_call)?;
                        return Self::model_validate_json(model, &out, validation_context);
                    }
                    None => Err(Error::Generic("No tool calls found".to_string())),
                }
            }
            IterableOrSingle::Iterable(_) => {
                match &message.tool_calls {
                    Some(tool_calls) => {
                        let tool_strings : Result<Vec<String>, Error>  = tool_calls
                            .iter()
                            .map(|tool_call| {
                                check_tool_call::<T>(tool_call)
                            }).collect::<Result<Vec<String>, Error>>();
                        
                        let merged_str = tool_strings?.join(",");
                        Self::model_validate_json(model, &merged_str, validation_context)
                    }
                    None => Err(Error::Generic("No tool calls found".to_string())),
                }
            }
        }
    }
}


fn validate_single<A, T>(data: T, validation_context: A) -> Result<T, Error> 
where
    T: ValidateArgs<'static, Args=A> + BaseSchema,
    A: BaseArg,
{
    match data.validate_args(validation_context) {  
        Ok(_) => Ok(data),
        Err(e) => Err(Error::ValidationErrors(e)),
    }
}

fn check_tool_call<T>(tool_call: &ChatCompletionMessageToolCall) -> Result<String, Error> 
{
    let tool_name = &tool_call.function.name;
    let model_name = type_name::<T>().split("::").last().unwrap();
    if tool_name != model_name {
        return Err(Error::Generic(format!(
            "tool call name: {} does not match model name: {}",
            tool_name, model_name
        )));
    }
    Ok(tool_call.function.arguments.clone())
}

