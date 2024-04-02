use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW;
use schemars::JsonSchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::enums::IterableOrSingle;


///we use rust macros to derive certain traits in order to serialize/deserialize format as json and Validate
#[derive(
    JsonSchema, serde::Serialize, Debug, Default, 
    validator::Validate, serde::Deserialize, Clone 
)]
struct Actor {
    ///We annotate the fields with the description of the field like you would do Field(..., description = "...") in pydantic
    #[schemars(description = "A string value representing the name of the person")]
    name : String,
    #[schemars(description = "The age of the actor")]
    age : i64,
    #[schemars(description = "3 movies the actor has been associated with")]
    
    ///we use the validate macros to validate specific fields 
    ///here we check that the movies vector has exactly 3 items
    #[validate(length(min = 3, max = 3, message = "movies must contain exactly 3 items"))]
    movies : Vec<String>,
}  


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            return an instance of an actor 
            ")),
            name: None,
        }],
    );
  
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Actor::default()),
        (),
        2,
        false, //consider removing this from the api, it appears streaming is not supported
        req,
    );
    println!("{:?}", result);
    Ok(())
}

