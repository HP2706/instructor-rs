use instructor_rs::traits::OpenAISchema;
use schemars::JsonSchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::{default, env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use validator::{Validate, ValidationError};
use openai_api_rs::v1::chat_completion::{Tool, ToolType, Function};
use instructor_rs::iterable::IterableOrSingle;
use instructor_rs::enums::InstructorResponse;
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};



fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let patched_client = Patch { client, mode: Some(Mode::TOOLS) };

    #[derive(JsonSchema, Serialize, Debug, Default, Deserialize, Clone)]
    enum TestEnum {
        #[default]
        PM,
        AM,
    }

    #[derive(
        JsonSchema, Serialize, Debug, Default, 
        Validate, Deserialize, Clone
    )]
    #[schemars(description = "this is a description of the weather api")]
    struct Weather {
        //#[schemars(description = "am or pm")]
        //time_of_day: TestEnum,
        #[schemars(description = "this is the hour from 1-12")]
        time: i64,
        city: String,
    }
    
    let call = true;
    println!("weather function: {:?}", Weather::tool_schema());

    if call {
        let req = ChatCompletionRequest::new(
            GPT4_TURBO_PREVIEW.to_string(),
            vec![chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(String::from("
                what is the weather at 10 in the evening in new york? 
                and what is the whether in the biggest city in Denmark in the evening?
                ")),
                name: None,
            }],
        );

        let result = patched_client.chat_completion(
            IterableOrSingle::Iterable(Weather::default()),
            (),
            1,
            false, //consider removing this from the api, it appears streaming is not supported
            req,
        );

        println!("result: {:?}", result);
        match result {
            Ok(response) => {
                match response {
                    InstructorResponse::One(e) => {
                        println!("InstructorResponse::Model {:?}", e);
                    }
                    InstructorResponse::Many(e) => {
                        println!("InstructorResponse::Many {:?}", e);
                    }
                }
            }
            Err(e) => {
                println!("got error: {:?}", e);
            }
        }
    }
    Ok(())
}


