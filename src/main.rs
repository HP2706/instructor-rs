use openai_api_rust::{Message, Role};
//internal libs
mod instructor;
mod types;
mod object;
mod traits;
mod utils;
use crate::instructor::chat_llm;
use crate::object::TestStruct;
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

