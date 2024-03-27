use openai_api_rust::{Message, Role};
//internal libs
mod instructor;
mod types;
mod object;
use crate::instructor::chat_llm;
use crate::object::TestStruct;

#[tokio::main]
async fn main() {
    let mut messages = vec![Message { role: Role::User, content: "Hello what is 7*10?".to_string() }];
    let answer = chat_llm::<TestStruct>(&mut messages).await;
    println!("{}", answer.value);
}
