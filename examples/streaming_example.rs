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
use async_stream::stream;
use instructor_rs::enums::InstructorResponse;
use futures::stream::{Stream, StreamExt, iter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

  
    ///we use rust macros to derive certain traits in order to serialize/deserialize format as json and Validate
    ///#[derive(
    ///  JsonSchema, Serialize, Debug, Default, 
    ///  Validate, Deserialize, Clone 
    ///)]
    #[schemars(description = "this is a description of the weather api")]
    #[derive(
        JsonSchema, Serialize, Debug, Default, 
        Validate, Deserialize, Clone
    )]
        
    struct Number {
        //#[schemars(description = "am or pm")]
        //time_of_day: TestEnum,
        #[schemars(description = "the value")]
        value: i64,
    }
    
    let req = CreateChatCompletionRequestArgs::default()
    .model(GPT4_TURBO_PREVIEW.to_string())
    .messages(vec![
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage{
                role: Role::User,
                content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                write 2 numbers, IT MUST BE 2 JSON OBJECTS
                ")),
                name: None,
            }
        )],
    )
    .stream(true)
    .model(GPT4_TURBO_PREVIEW.to_string())
    .build().unwrap();


    let result = patched_client.chat_completion(
        ///we wrap in an Iterable enum to allow more than one function call 
        /// a bit like List[Type[BaseModel]] or Iterable[Type[BaseModel]] in instructor
        IterableOrSingle::Iterable(Number::default()),
        (),
        1,
        req,
    );

    use std::time::Instant;


    let model = result.await;
    match model {
        Ok(x) => {
            match x {
                InstructorResponse::Many(x) => println!("result: {:?}", x),
                InstructorResponse::One(x) => println!("result: {:?}", x),
                InstructorResponse::Stream(mut x) => {
                    let t0 = Instant::now();
                    while let Some(x) = x.next().await {
                        println!("main!! result: {:?} at time {:?}", x, t0.elapsed());
                    }
                },
            }
        }
        Err(e) => println!("error: {:?}", e),
    }
    Ok(())
}


