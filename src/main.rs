/* use serde::{Deserialize, Serialize};
use validator::Validate;// Import serde_json Error
use schemars::JsonSchema;

use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW; 
use std::{env, vec};

use internal_macros::*;

use instructor_rs::patch::{Patch, InstructorChatCompletionCreate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());

   /*  let instructor_client = Patch { client }; */

    #[derive(Debug, Deserialize, Serialize, JsonSchema, Validate, OpenAISchema)]
    struct ResponseModel {
        message: String,
    }

    ResponseModel::from_response(&response, ResponseModel::Args, Mode::JSON);

    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("What is Bitcoin?")),
            name: None,
        }],
    );
    let result = client.chat_completion(req)?;
    println!("{:?}", result.choices[0].message.content);


    Ok(())
}
*/

pub mod test;
pub mod utils;
use crate::test::{Mode, Error, extract_json_from_codeblock};

fn main() {
    #[derive(JsonSchema, serde::Serialize, Debug, Default, validator::Validate, serde::Deserialize)]
    struct Test {
        #[validate(custom(function = "validate", arg = "(i64)"))]
        value: i64,
    }

    use validator::ValidationError;

    fn validate(value: i64, arg: i64) -> Result<(), ValidationError> {
        if value > arg {
            return Err(ValidationError::new("Value is greater than arg"));
        }
        Ok(())
    }
  
    
    
    let data = "{
        \"value\": 6
    }";
    match Test::model_validate_json(data, (5)) {
        Ok(a) => println!("{:?}", a),
        Err(e) => println!("{:?}", e),
    }
}


