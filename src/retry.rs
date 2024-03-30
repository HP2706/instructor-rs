use serde::{Serialize, Deserialize};
use crate::mode::Mode;
use crate::traits::OpenAISchema;
use crate::enums::Error;
use crate::process_response::process_response;
use schemars::JsonSchema;
use validator::ValidateArgs;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionMessage, MessageRole, Content
};
use std::fmt;
use openai_api_rs::v1::error::APIError;
use crate::enums::InstructorResponse;
use crate::enums::IterableOrSingle;

pub fn reask_messages(
    response: &ChatCompletionResponse, mode: Mode, exception: impl fmt::Display
) -> impl Iterator<Item = ChatCompletionMessage> {

    let first_message = &response.choices[0].message;
    let message = ChatCompletionMessage {
        role: first_message.role.clone(),
        content: Content::Text(first_message.content.clone().unwrap()),
        name: None,
    };
    let mut messages: Vec<ChatCompletionMessage> = vec![message];
    //TODO fix this
    match mode {
        Mode::MD_JSON => {
            messages.push(
                ChatCompletionMessage {
                    role: MessageRole::user,
                    content: Content::Text(format!(
                        "Correct your JSON ONLY RESPONSE, based on the following errors:\n{}",
                        exception
                    )),
                    name: None,
                }
            );
        }
        _ => {
            messages.push(
                ChatCompletionMessage {
                    role: MessageRole::user,
                    content: Content::Text(format!(
                        "Recall the function correctly, fix the errors, exceptions found\n{}",
                        exception
                    )),
                    name: None,
                }
            );
        }
    }

    messages.into_iter()
}


pub fn retry_sync<'v_a, 'f, T, A>(
    func: Box<dyn Fn(ChatCompletionRequest) -> Result<ChatCompletionResponse, APIError> + 'f>,
    response_model: Option<IterableOrSingle<T>>,
    validation_context: Option<A>,
    kwargs : &mut ChatCompletionRequest, 
    max_retries: usize,
    mode: Mode,
) -> Result<InstructorResponse<A, T>, Error>
where
    T: ValidateArgs<'static, Args=A> + Serialize + for<'de> Deserialize<'de> + JsonSchema + OpenAISchema<A, T>,
    A: 'static + Copy,
{
    let mut attempt = 0;

    while attempt < max_retries {
        let response = &func(kwargs.clone());
        match response {
            Ok(response) => {
                let result = process_response(
                    response, &response_model, false, validation_context.as_ref().unwrap(), mode
                );

                match result {
                    Ok(result) => {
                        match result {
                            IterableOrSingle::Single(item) => return Ok(InstructorResponse::Model(item)),
                            IterableOrSingle::Iterable(items) => {
                            },
                        }
                    }
                    Err(e) => {
                        let messages = reask_messages(&response, mode, e);
                        println!("number of messages before: {}", kwargs.messages.len());
                        kwargs.messages.extend(messages);
                        println!("number of messages after: {}", kwargs.messages.len());
                        attempt += 1;
                    }
                }
            }
            Err(e) => {
                return Err(Error::Generic(format!("Error: {}", e).to_string()));
            }
        }
    }
    Err(Error::Generic("Max retries exceeded".to_string()))
}
