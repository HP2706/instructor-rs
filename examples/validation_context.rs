use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW;
use schemars::JsonSchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use validator::ValidationError;
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::iterable::IterableOrSingle;
use serde::{Serialize, Deserialize};
use model_traits_macro::derive_all;
use validator::Validate;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            return an instance of an director that is more than 60 years old (hint steven spielberg)
            ")),
            name: None,
        }],
    );
  
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Director::default()),
        (2024-60),
        2,
        false, //consider removing this from the api, it appears streaming is not supported
        req,
    );
    println!("{:?}", result);
    /// Ok(InstructorResponse::Single({ name: "Steven Spielberg", age: 77, birth_year: 1946 }))
    Ok(())
}

