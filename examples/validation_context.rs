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


#[derive_all]
///we use rust macros to derive certain traits in order to serialize/deserialize format as json and Validate
///#[derive(
///  JsonSchema, Serialize, Debug, Default, 
///  Validate, Deserialize, Clone 
///)]
struct Director {
    ///We annotate the fields with the description of the field like you would do Field(..., description = "...") in pydantic
    #[schemars(description = "A string value representing the name of the person")]
    name : String,
    
    #[schemars(description = "The age of the director, the age of the director must be a multiple of 3")]
    #[validate(custom(function = "check_is_multiple", arg = "i64"))]
    ///we define custom validation function that can take in foreign input and perform validation logic based on input
    age : i64,
    #[schemars(description = "year of birth")] 
    birth_year : i64
}  

fn check_is_multiple(age: i64, arg : i64) -> Result<(), ValidationError> {
    if age % 3 == 0 {
        Ok(())
    } else {
        Err(ValidationError::new("The age {} is not a multiple of 3"))
    }
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
                return an instance of an director that is more than 60 years old (hint steven spielberg)
                ")),
                name: None,
            }
        )],
    ).build().unwrap();

    ///we wrap in an Iterable enum to allow more than one function call 
    /// a bit like List[Type[BaseModel]] or Iterable[Type[BaseModel]] in instructor
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Director::default()),
        (2024-60),
        2,
        req,
    );

    println!("result: {:?}", result.await);
    /// Ok(InstructorResponse::Single({ name: "Steven Spielberg", age: 77, birth_year: 1946 }))
    Ok(())
}


