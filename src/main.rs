use instructor_rs::dsl::iterable::IterableBase;
use schemars::JsonSchema;
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::{Patch};
use instructor_rs::enums::IterableOrSingle;
use model_traits_macro::derive_all;
use serde::{Deserialize, Serialize};
use validator::Validate;
use instructor_rs::common::{GPT3_5_TURBO, GPT4_TURBO_PREVIEW};
use async_openai::types::{
    CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    ChatCompletionRequestUserMessage, ChatCompletionRequestMessage, Role,
    ChatCompletionRequestUserMessageContent, ChatCompletionTool, ChatCompletionToolType,
    ChatCompletionResponseStream, ChatCompletionRequestAssistantMessage
};
use async_openai::error::OpenAIError;
use async_openai::Client;
use instructor_rs::enums::{InstructorResponse, ChatCompletionResponseWrapper};
use instructor_rs::types::JsonStream;
use futures::stream::{Stream, StreamExt, iter};
use pin_utils::pin_mut;
use std::time::Instant;
use async_stream::stream;
use instructor_rs::utils::{create_chat_completion_stream, string_to_stream};
use instructor_rs::retry::retry_async;
use instructor_rs::process_response::handle_response_model;
use futures::Future;
use std::error::Error;
use std::pin::Pin;


#[derive_all]
#[schemars(description = "this is a number api")]
struct Number {
    //#[schemars(description = "am or pm")]
    //time_of_day: TestEnum,
    a: i64,
    b: i64,
    #[schemars(description = "operation to perform must be one of: add, subtract, multiply, divide")]
    operation: String
}

async fn test(){
    let stream = vec![
        Number { a: 1, b: 2, operation: "add".to_string() },
        Number { a: 3, b: 4, operation: "subtract".to_string() },
        Number { a: 5, b: 6, operation: "multiply".to_string() },
        Number { a: 7, b: 8, operation: "divide".to_string() },
    ]
    .iter()
    .map(|x| 
        //sleep 0.1 seconds
        {
            serde_json::to_string(x).unwrap()
        }
    )
    .collect::<Vec<String>>().join(",");
        
    let stream = string_to_stream(stream).await;
    let stream = create_chat_completion_stream(stream).await;

    let t0 = Instant::now();
    let out = Number::from_streaming_response_async(
        IterableOrSingle::Iterable(Number::default()),
        stream,
        &(),
        Mode::TOOLS
    ).await;

    match out {
        InstructorResponse::Many(x) => println!("result: {:?}", x),
        InstructorResponse::One(x) => println!("result: {:?}", x),
        InstructorResponse::Stream(x) => {
            pin_mut!(x);
            while let Some(x) = x.next().await {
                println!("main!! result: {:?} at time {:?}", x, t0.elapsed());
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mode = Mode::TOOLS;
    let patched_client = Patch { client: client.clone(), mode: Some(mode) };
    
    

    let test = true;
    if test {
        let mut req = CreateChatCompletionRequestArgs::default()
        .model(GPT3_5_TURBO.to_string())
        .messages(vec![
            ChatCompletionRequestMessage::Assistant(
                ChatCompletionRequestAssistantMessage{
                    role: Role::Assistant,
                    content: Some("
                    return JSON for each of the following questions:
                    in the following schema {}
                    1. what is 10 times 10?
                    2. what is 10 plus 10?
                    ".to_string()),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                },
            ),
        ])
        .stream(true)
        .build()
        .unwrap();
            
        let t0 = Instant::now();
        let result = patched_client.chat_completion(
            IterableOrSingle::Iterable(Number::default()),
            (),
            1,
            req
        );

        let x = result.await.unwrap();
        
        match x {
            InstructorResponse::Many(x) => println!("result: {:?}", x),
            InstructorResponse::One(x) => println!("result: {:?}", x),
            InstructorResponse::Stream(x) => {
                pin_mut!(x);
                while let Some(x) = x.next().await {
                    println!("main!! result: {:?} at time {:?}", x, t0.elapsed());
                }
            },
        }
    }
    Ok(())
}


