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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::TOOLS) };

    #[derive(JsonSchema, Serialize, Debug, Default, Deserialize, Clone)] 
    ///we cannot use #[derive_all] here as enums cannot derive Validate Trait
    enum TestEnum {
        #[default]
        PM,
        AM,
    }

    #[derive_all]
    ///we use rust macros to derive certain traits in order to serialize/deserialize format as json and Validate
    ///#[derive(
    ///  JsonSchema, Serialize, Debug, Default, 
    ///  Validate, Deserialize, Clone 
    ///)]
    #[schemars(description = "this is a description of the weather api")]
    struct Weather {
        //#[schemars(description = "am or pm")]
        //time_of_day: TestEnum,
        #[schemars(description = "this is the hour from 1-12")]
        time: i64,
        city: String,
    }
    
    let req = CreateChatCompletionRequestArgs::default()
    .model(GPT4_TURBO_PREVIEW.to_string())
    .messages(vec![
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage{
                role: Role::User,
                content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                what is the weather at 10 in the evening in new york? 
                and what is the whether in the biggest city in Denmark in the evening?
                return in specified json format
                ")),
                name: None,
            }
        )],
    ).build().unwrap();

    let result = patched_client.chat_completion(
        ///we wrap in an Iterable enum to allow more than one function call 
        /// a bit like List[Type[BaseModel]] or Iterable[Type[BaseModel]] in instructor
        IterableOrSingle::Iterable(Weather::default()),
        (),
        1,
        req,
    );

    println!("result: {:?}", result.await);
    ///Ok(Many([Weather { time: 10, city: "New York" }, Weather { time: 10, city: "Copenhagen" }]))
    
    Ok(())
}


