use openai_api_rust::{Message, Role};
//internal libs
mod instructor;
mod types;
mod object;
use crate::instructor::chat_llm;
use crate::object::SignupData;

#[tokio::main]
async fn main() {
    let mut messages = vec![Message { role: Role::User, content: "Hello!".to_string() }];
    let answer = chat_llm::<SignupData>(&mut messages).await;
    println!("{}", answer.value);
}

