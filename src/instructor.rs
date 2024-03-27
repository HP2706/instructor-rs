use serde::Serialize;
use dotenvy::dotenv;
use openai_api_rust::*;
use openai_api_rust::chat::*;
use openai_api_rust::completions::*;
use schemars::{JsonSchema, schema_for};
use crate::object::JsonLoad;
use validator::Validate;

pub async fn chat_llm<T>(messages: &mut Vec<Message>) -> T
    where T: 
        JsonSchema + for<'a> validator::ValidateArgs<'a> + 
        JsonLoad + Validate + Default 
    {
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");

    
    let schema = schema_for!(T);
    let schema_string = serde_json::to_string_pretty(&schema).unwrap();
    println!("schema string is: {}", schema_string);
    
    let message = Message {
        role: Role::User,
        content: "You must return your response in the schema for the data model: \n\n".to_string() + &schema_string,
    };

    messages.push(message);
    
    let body = ChatBody {
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: None,
        temperature: None,
        top_p: None,
        n: None,
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: messages.clone(),
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();
    println!("message: {:?}", message.content);
    let data = T::load_from_json(&message.content).unwrap();
    return data
}

