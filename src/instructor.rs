use serde::Serialize;
use dotenvy::dotenv;
use openai_api_rust::*;
use openai_api_rust::chat::*;
use openai_api_rust::completions::*;
use schemars::{JsonSchema, schema_for};
use crate::utils::validate_with_args;
use crate::traits::LoadFromJson;
use validator::{Validate, ValidateArgs};

pub async fn chat_llm<'v_a, T>(messages: &mut Vec<Message>, validation_args: T::Args) 
    -> T
    where T: 
        JsonSchema + Validate + Default + ValidateArgs<'v_a> + LoadFromJson<T> + Serialize
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
    println!("messages: {:?}", messages);
    
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
    return validate_with_args(data, validation_args).unwrap();
}

