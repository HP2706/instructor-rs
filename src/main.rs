use schemars::JsonSchema;
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::iterable::IterableOrSingle;
use model_traits_macro::derive_all;
use serde::{Deserialize, Serialize};
use validator::Validate;
use instructor_rs::common::GPT4_TURBO_PREVIEW;
use async_openai::types::{
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    ChatCompletionRequestUserMessage, ChatCompletionRequestMessage, Role,
    ChatCompletionRequestUserMessageContent
};
use async_openai::Client;

#[derive_all]
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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    let req = CreateChatCompletionRequestArgs::default()
        .model(GPT4_TURBO_PREVIEW.to_string())
        .messages(vec![
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage{
                    role: Role::User,
                    content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                    return an instance of an actor 
                    ")),
                    name: None,
                }
            )],
        ).build().unwrap();
  
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Actor::default()),
        (),
        2,
        false, //consider removing this from the api, it appears streaming is not supported
        req,
    );
    println!("{:?}", result.await);
    ///Ok(InstructorResponse::Single(
    /// Actor { name: "Leonardo DiCaprio", 
    /// age: 49, 
    /// movies: vec![String::from("Django unchained"), String::from("Once upon a time in holywood"), String::from("Titanic")] })
    /// )
    Ok(())
}

