/* use openai_api_rust::{Message, Role};
//internal libs
use instructor_rs::instructor::chat_llm;
use instructor_rs::object::TestStruct;
use std::borrow::BorrowMut;
use std::future::Future;
use std::pin::Pin;

#[tokio::main]
async fn main(){
    
    let mut messages = vec![
        Message {
            role: Role::User,
            content: "You must return your response in the schema for the data model what is the product of 2 * 3: \n\n".to_string(),
        }
    ];
    let data = chat_llm::<TestStruct>(messages.borrow_mut(), (2, 3)).await;

    //println!("data: {:?}", data);
}
*/
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW; 
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());

    let mut properties = HashMap::new();
    properties.insert(
        "coin".to_string(),
        Box::new(chat_completion::JSONSchemaDefine {
            schema_type: Some(chat_completion::JSONSchemaType::String),
            description: Some("The cryptocurrency to get the price of".to_string()),
            ..Default::default()
        }),
    );
    
    #[derive(Debug, Deserialize, Serialize)]
    struct CoinPrice {
        name: String,
        price: f64,
    }



    let mut json = HashMap::from([("type".to_string(), "json_object".to_string())]);
    let json_value = serde_json::to_value(json).unwrap();

    let value = serde_json::to_value(&CoinPrice {
        name: "btc".to_string(),
        price: get_coin_price("btc"),
    })?;

    println!("{:?}", json_value);
    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: chat_completion::Content::Text(String::from("Return in the same schema format but for ethereum, guess the price")),
                name: None,
            }, 
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::assistant,
                content: chat_completion::Content::Text(String::from(value.to_string())),
                name: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::assistant,
                content: chat_completion::Content::Text(String::from("this is an example of json you should return: \n\n") + &value.to_string()),
                name: None,
            },

        ],
    ).response_format(json_value).stream(false).max_tokens(100).temperature(0.0);
    
    let response = client.chat_completion(req)?;
    println!("{:?}", response.choices[0].message.content);
    // debug request json
    // let serialized = serde_json::to_string(&req).unwrap();
    // println!("{}", serialized);

    Ok(())
}
