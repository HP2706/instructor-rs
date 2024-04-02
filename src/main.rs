use validator::ValidationError;// Import serde_json Error
use schemars::JsonSchema;
use instructor_rs::traits::OpenAISchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::enums::IterableOrSingle;
use instructor_rs::enums::InstructorResponse;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());

    let patched_client = Patch { client, mode: Some(Mode::JSON) };
   /*  let instructor_client = Patch { client }; */
  
   
    #[derive(
        JsonSchema, serde::Serialize, Debug, Default, 
        validator::Validate, serde::Deserialize, Clone, Copy
    )]
    struct Number {
        //value: description
        #[schemars(description = "An integer value")]
        value1: i64,

    }        
    

    let req = ChatCompletionRequest::new(
        "gpt-3.5-turbo-0125".to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            i want you to produce 10 examples in the expected format each should be a Number. return like this 
            {value1 : x}, {value1 : y}, {value1 : z}...
            ")),
            name: None,
        }],
    );


    let debug = false;

    if !debug {
        let result = patched_client.chat_completion(
            IterableOrSingle::Iterable(Number::default()),
            (),
            2,
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


