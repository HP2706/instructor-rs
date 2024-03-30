use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};// Import serde_json Error
use schemars::JsonSchema;
use instructor_rs::traits::OpenAISchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW; 
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::enums::IterableOrSingle;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());

    let patched_client = Patch { client, mode: Some(Mode::JSON) };
   /*  let instructor_client = Patch { client }; */

    #[derive(JsonSchema, serde::Serialize, Debug, Default, validator::Validate, serde::Deserialize, Clone)]
    struct TestStruct {
        #[validate(custom(function = "validate", arg = "( i64)"))]
        value: i64,
    }

    fn validate(value: i64, arg: i64) -> Result<(), ValidationError> {
        if value < 0 {
            return Err(ValidationError::new("value must be greater than 0"));
        }
        Ok(())
    }


    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("What is Bitcoin?")),
            name: None,
        }],
    );
    
    let result = patched_client.chat_completion(
        Some(IterableOrSingle::Iterable(TestStruct::default())),
        Some(1),
        1,
        req
    );
    


    Ok(())
}


