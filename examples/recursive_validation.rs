use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW;
use schemars::JsonSchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use validator::ValidationError;
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::enums::IterableOrSingle;

///we use rust macros to derive certain traits in order to serialize/deserialize format as json and Validate
#[derive(
    JsonSchema, serde::Serialize, Debug, Default, 
    validator::Validate, serde::Deserialize, Clone 
)]
struct Movies {
    /* #[schemars(description = "A list of movies that are bloody and or rough")]
    #[validate(length(min = 5, message = "movies must contain exactly 5 items"))]
    bloody_movies : Vec<String>,
    #[schemars(description = "A list of movies that are soft and or romantic")] */
    #[validate(length(min = 5, message = "movies must contain exactly 5 items"))]
    #[validate(custom(function = "check_are_soft"))]
    soft_movies : Vec<String>,
}


#[derive(
    JsonSchema, serde::Serialize, Debug, Default, 
    validator::Validate, serde::Deserialize, Clone 
)]
struct IsSoft {
    #[schemars(description = "A boolean value that indicates whether all movies are soft and or romantic")]
    content : bool
}


fn check_are_soft(value: &Vec<String>) -> Result<(), ValidationError> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            return true if all movies are soft and or romantic, false otherwise in the specified json format
            ")),
            name: None,
        }],
    );
  
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(IsSoft::default()),
        (), //no validation context needed
        2,
        false, //consider removing this from the api, it appears streaming is not supported
        req,
    );
    
    match result {
        Ok(res) => {
            match res.unwrap().content {
                true => return Ok(()),
                false => return Err(ValidationError::new("movies are not soft and or romantic")),
            }
        }
        Err(e) => {
            ///if the llm fails, the movies are undecisive and we reject them
            return Err(ValidationError::new("movies are undecisive"))
        }
        
    
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            return 5 movies that are soft and or romantic
            ")),
            name: None,
        }],
    );
  
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Movies::default()),
        (), //no validation context needed
        2,
        false, //consider removing this from the api, it appears streaming is not supported
        req,
    );
    println!("{:?}", result);
    Ok(())
}

