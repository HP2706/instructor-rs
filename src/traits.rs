use schemars::JsonSchema;
use validator::ValidateArgs;
use serde::{Deserialize, Serialize};
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::enums::{
    Error,
    InstructorResponse,
    IterableOrSingle
};
use crate::mode::Mode;
use crate::utils::extract_json_from_codeblock;
use std::fmt::Debug;

pub trait BaseSchema<T>: 'static + Debug + Copy + Serialize + for<'de> Deserialize<'de> + ValidateArgs<'static> + JsonSchema + Sized {}

impl<T> BaseSchema<T> for T
where T: 'static + Copy + Debug + Serialize + for<'de> Deserialize<'de> + ValidateArgs<'static> + JsonSchema + Sized
{}

pub trait OpenAISchema<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + BaseSchema<T>,
    Args: 'static + Copy,
{
    type Args : 'static + Copy;
    fn openai_schema() -> String; 
 
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
        println!("text in parse_json: {:?}", text);
        let json_extract = extract_json_from_codeblock(&text);
        println!("json_extract: {:?}", json_extract);
        match json_extract {
            Ok(json_extract) => {
                return Self::model_validate_json(model, &json_extract, validation_context);
            }
            Err(e) => {
                return Err(e);
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


/* pub trait IterableBase<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + OpenAISchema<Args, T> + BaseSchema<T>,
    Args: 'static + Copy,
{
    type Args: 'static + Copy;

    fn tasks_from_chunks(

    )
} */