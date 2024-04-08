use instructor_rs::utils::to_sync;
use schemars::JsonSchema;
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::enums::IterableOrSingle;
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
use validator::ValidationError;
use instructor_rs::enums::InstructorResponse;

#[derive_all]
struct Movies {
    #[validate(length(min = 5, message = "movies must contain exactly 5 items"))]
    #[validate(custom(function = "check_are_soft"))]
    soft_movies : Vec<String>,
}

#[derive_all]
struct IsSoft {
    #[schemars(description = "A boolean value that indicates whether all movies are soft and or romantic")]
    content : bool
}

#[derive_all]
#[schemars(description = "this is a description of the weather api")]
    struct Weather {
        //#[schemars(description = "am or pm")]
        //time_of_day: TestEnum,
        #[schemars(description = "this is the hour from 1-12")]
        time: i64,
        city: String,
    }
    

///we define our validation function for the soft_movies field
/// This function calls an llm and checks if all movies are soft and or romantic
fn check_are_soft(value: &Vec<String>) -> Result<(), ValidationError> {
    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::JSON) };
    tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
        let req = CreateChatCompletionRequestArgs::default()
        .model(GPT4_TURBO_PREVIEW.to_string())
        .messages(vec![
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage{
                    role: Role::User,
                    content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                    return true if all movies are soft and or romantic, false otherwise in the specified json format
                    ")),
                    name: None,
                }
            )],
        ).build().unwrap();
      
        let future = patched_client.chat_completion(
            IterableOrSingle::Single(IsSoft::default()),
            (), //no validation context needed
            2,
            req,
        );
    
    
        let result =to_sync(future);
        
        match result {
            Ok(res) => {
                match res.unwrap().unwrap().content {
                    true => return Ok(()),
                    false => return Err(ValidationError::new("movies are not soft and or romantic")),
                }
            }
            Err(e) => {
                ///if the llm fails, the movies are undecisive and we reject them
                return Err(ValidationError::new("movies are undecisive"))
            }
        }
    })
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
                return 5 movies that are soft and or romantic
                ")),
                name: None,
            }
        )],
    ).build().unwrap();

   
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Movies::default()),
        (), //no validation context needed
        2,
        req,
    );
    println!("{:?}", result.await);
    Ok(())
}




//fn main(){}