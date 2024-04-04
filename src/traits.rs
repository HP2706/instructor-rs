use schemars::JsonSchema;
use validator::ValidateArgs;
use serde::{Deserialize, Serialize};
use openai_api_rs::v1::chat_completion::{ChatCompletionResponse, ToolCall};
use crate::enums::{
    Error, InstructorResponse
};
use openai_api_rs::v1::chat_completion::{
    Function, FunctionParameters, JSONSchemaDefine, JSONSchemaType
};

use crate::iterable::IterableOrSingle;
use crate::mode::Mode;
use crate::utils::extract_json_from_codeblock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::any::type_name;


pub trait BaseSchema<T>: 'static + Debug + Serialize + for<'de> Deserialize<'de> + ValidateArgs<'static> + JsonSchema + Sized {}

impl<T> BaseSchema<T> for T
where T: 'static + Debug + Serialize + for<'de> Deserialize<'de> + ValidateArgs<'static> + JsonSchema + Sized {}



pub trait OpenAISchema<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + BaseSchema<T>,
    Args: 'static + Copy,
{
    type Args : 'static + Copy;
    fn openai_schema() -> String; 

    fn tool_schema() -> Function;
 
    fn model_validate_json(
        model : &IterableOrSingle<Self>, 
        data: &str, 
        validation_context: &Args
    ) -> Result<InstructorResponse<Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;
    
    fn from_response(
        model: &IterableOrSingle<Self>,
        response: &ChatCompletionResponse,
        validation_context: &Args,
        mode: Mode,
    ) -> Result<InstructorResponse<Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;
    
    fn parse_json(
        model: &IterableOrSingle<Self>,
        completion: &ChatCompletionResponse,
        validation_context: &Args,
    ) -> Result<InstructorResponse<Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;

    fn parse_tools(
        model: &IterableOrSingle<Self>,
        completion: &ChatCompletionResponse,
        validation_context: &Args,
    ) -> Result<InstructorResponse<Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;

}

impl<A, T> OpenAISchema<A, T> for T
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    type Args = A;

    // The rest of your implementation remains the same...

    fn openai_schema() -> String where T: serde::Serialize + JsonSchema {
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_value(&schema).unwrap();
    
        // Extracting basic schema information
        let title = schema_json["title"].as_str().unwrap_or_default().to_string();
        let description = format!("Correctly extracted `{}` with all the required parameters with correct types", title);
    
        // Transforming definitions to match the desired format
        let definitions = schema_json["definitions"].as_object().unwrap_or(&serde_json::Map::new()).clone();
        let mut defs = serde_json::Map::new();
        for (key, value) in definitions.iter() {
            let mut def = value.clone();
            if let Some(props) = def["properties"].as_object_mut() {
                for (prop_key, prop_value) in props.iter_mut() {
                    if let Some(schemars_desc) = prop_value["metadata"]["description"].take().as_str() {
                        prop_value["description"] = serde_json::Value::String(schemars_desc.to_string());
                    }
                }
            }
            defs.insert(key.clone(), def);
        }
    
        // Transforming properties to match the desired format
        let properties = schema_json["properties"].as_object().unwrap_or(&serde_json::Map::new()).clone();
        let mut parameters = serde_json::Map::new();
        for (key, value) in properties.iter() {
            let mut param = value.clone();
            if let Some(schemars_desc) = param["metadata"]["description"].take().as_str() {
                param["description"] = serde_json::Value::String(schemars_desc.to_string());
            }
            parameters.insert(key.clone(), param);
        }
    
        // Required fields
        let required = schema_json["required"].as_array().unwrap_or(&Vec::new()).clone();
    
        // Constructing the final schema in the desired format
        let final_schema = serde_json::json!({
            "name": title,
            "description": description,
            "parameters": {
                "$defs": defs,
                "properties": parameters,
                "required": required,
                "type": "object"
            }
        });
    
        serde_json::to_string_pretty(&final_schema).unwrap()
    }

    fn tool_schema() -> Function {
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_value(&schema).unwrap();
        let title = schema_json["title"].as_str().unwrap_or_default().to_string();
    
        let description = match schema_json["description"].as_str() {
            Some(desc) => desc.to_string(),
            None => format!("Correctly extracted `{}` with all the required parameters with correct types", title),
        };
    
        let mut properties: HashMap<String, Box<JSONSchemaDefine>> = HashMap::new();
        let mut required: Vec<String> = Vec::new();
    
        if let Some(props) = schema_json["properties"].as_object() {
            for (prop_name, prop_value) in props {
                let mut prop_schema = JSONSchemaDefine {
                    schema_type: prop_value["type"].as_str().map(|t| match t {
                        "string" => JSONSchemaType::String,
                        "integer" => JSONSchemaType::Number,
                        "float" => JSONSchemaType::Number, 
                        "number" => JSONSchemaType::Number,
                        "boolean" => JSONSchemaType::Boolean,
                        "array" => JSONSchemaType::Array,
                        "object" => JSONSchemaType::Object,
                        _ => JSONSchemaType::String,
                    }),
                    description: prop_value["description"].as_str().map(|d| d.to_string()),
                    ..Default::default()
                };
    
                if let Some(all_of) = prop_value["allOf"].as_array() {
                    if let Some(ref_value) = all_of[0]["$ref"].as_str() {
                        if let Some(def_name) = ref_value.split('/').last() {
                            if let Some(def_value) = schema_json["definitions"][def_name].as_object() {
                                if let Some(enum_values) = def_value["enum"].as_array() {
                                    prop_schema.enum_values = Some(enum_values.iter().map(|v| v.as_str().unwrap().to_string()).collect());
                                }
                            }
                        }
                    }
                }
    
                properties.insert(prop_name.to_string(), Box::new(prop_schema));
            }
        }
    
        if let Some(req) = schema_json["required"].as_array() {
            required = req.iter().map(|r| r.as_str().unwrap().to_string()).collect();
        }
    
        Function {
            name: title,
            description: Some(description),
            parameters: FunctionParameters {
                schema_type: JSONSchemaType::Object,
                properties: Some(properties),
                required: Some(required),
            },
        }
    }

    fn model_validate_json(
        model: &IterableOrSingle<Self>, 
        data: &str, 
        validation_context: &Self::Args
    ) -> Result<InstructorResponse<Self::Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>,
    {

        match model {
            IterableOrSingle::Iterable(_) => {
                let bracketed_data = &format!("[{}]", data);
                let data = serde_json::from_str::<Vec<T>>(bracketed_data)
                    .map_err(|e| Error::SerdeError(e)); // Convert serde_json::Error to your custom Error::SerdeError
                data.and_then(|data| {
                    data.into_iter().map(|item| validate_single(item, *validation_context))
                        .collect::<Result<Vec<T>, Error>>() 
                        .map(|data| InstructorResponse::Many(data)) 
                })
            },
            IterableOrSingle::Single(_) => {
                let data = serde_json::from_str::<T>(data);
                match data {
                    Ok(data) => {
                        let validated_data = validate_single(data, *validation_context); 
                        validated_data.map(|data| InstructorResponse::One(data)) 
                    },
                    Err(e) => Err(Error::SerdeError(e)),
                }
            }
        }
    }

    fn from_response(
        model: &IterableOrSingle<Self>,
        response: &ChatCompletionResponse,
        validation_context: &Self::Args,
        mode: Mode,
    ) -> Result<InstructorResponse<Self::Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>,
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

    fn parse_json(
        model: &IterableOrSingle<Self>,
        completion: &ChatCompletionResponse,
        validation_context: &Self::Args,
    ) -> Result<InstructorResponse<Self::Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>,
    {
        let text = completion.choices[0].message.content.clone().unwrap();
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

    fn parse_tools(
        model: &IterableOrSingle<Self>,
        completion: &ChatCompletionResponse,
        validation_context: &Self::Args,
    ) -> Result<InstructorResponse<Self::Args, T>, Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>,
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
                        
                        match tool_strings {
                            Ok(tool_strings) => {
                                //we merge string into one str separted by comma
                                let merged_str = tool_strings.join(",");
                                Self::model_validate_json(model, &merged_str, validation_context)
                            }
                            Err(e) => Err(e),
                        }
                    
                    }
                    None => Err(Error::Generic("No tool calls found".to_string())),
                }
            }
        }
    }
}


fn validate_single<A, T>(data: T, validation_context: A) -> Result<T, Error> 
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    match data.validate_args(validation_context) {  
        Ok(_) => Ok(data),
        Err(e) => Err(Error::ValidationErrors(e)),
    }
}

fn check_tool_call<T>(tool_call: &ToolCall) -> Result<String, Error> 
{
    let tool_name = tool_call.function.name.as_ref().unwrap();
    let model_name = type_name::<T>().split("::").last().unwrap();
    if tool_name != model_name {
        return Err(Error::Generic(format!(
            "tool call name: {} does not match model name: {}",
            tool_name, model_name
        )));
    }
    if tool_call.function.arguments.as_ref().is_none() {
        return Err(Error::Generic(format!("tool call arguments are empty in tool {:?}", tool_call)));
    }
    Ok(tool_call.function.arguments.as_ref().unwrap().to_string())
}

