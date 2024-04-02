use crate::mode::Mode;
use crate::enums::Error;
use crate::process_response::process_response;
use crate::traits::BaseSchema;
use validator::ValidateArgs;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionMessage, MessageRole, Content
};
use std::fmt;
use openai_api_rs::v1::error::APIError;
use crate::enums::InstructorResponse;
use crate::enums::IterableOrSingle;

pub fn reask_messages(
    response: &ChatCompletionResponse,
    mode: Mode,
    exception: impl fmt::Display,
) -> Vec<ChatCompletionMessage> {
    let first_message = &response.choices[0].message;
    let message = ChatCompletionMessage {
        role: first_message.role.clone(),
        content: Content::Text(first_message.content.clone().unwrap()),
        name: None,
    };
    let mut messages: Vec<ChatCompletionMessage> = vec![message];

    match mode {
        Mode::MD_JSON => {
            messages.push(ChatCompletionMessage {
                role: MessageRole::user,
                content: Content::Text(format!(
                    "Correct your JSON ONLY RESPONSE, based on the following errors:\n{}\n",
                    exception
                )),
                name: None,
            });
        }
        _ => {
            messages.push(ChatCompletionMessage {
                role: MessageRole::user,
                content: Content::Text(format!(
                    "Recall the function correctly, fix the errors, exceptions found\n{}",
                    exception
                )),
                name: None,
            });
        }
    }

    messages
}

pub fn retry_sync<'f, T, A>(
    func: Box<dyn Fn(ChatCompletionRequest) -> Result<ChatCompletionResponse, APIError> + 'f>,
    response_model: IterableOrSingle<T>,
    validation_context: A,
    kwargs: &mut ChatCompletionRequest,
    max_retries: usize,
    stream : bool,
    mode: Mode,
) -> Result<InstructorResponse<A, T>, Error>
where
    T: ValidateArgs<'static, Args = A> + BaseSchema<T>,
    A: 'static + Copy,
{
    let mut attempt = 0;

    while attempt < max_retries {
        println!("message to model\n\n {:?}", kwargs.messages);
        println!("attempt: {}", attempt);
        let response = func(kwargs.clone());
        match response {
            Ok(_response) => {
                println!("model responded with: {}", _response.choices[0].message.content.clone().unwrap());
                let result = process_response(
                    &_response,
                    &response_model,
                    stream,
                    &validation_context,
                    mode,
                );

                match result {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        println!("Error: {}", e);
                        let messages = reask_messages(&_response, mode, e);
                        println!("number of messages before: {}", kwargs.messages.len());
                        kwargs.messages.extend(messages);
                        println!("number of messages after: {}", kwargs.messages.len());
                        attempt += 1;
                        continue;
                    }
                }
            }
            Err(e) => {
                println!("retry_sync Error: {}", e);
                return Err(Error::Generic(format!("Error: {}", e).to_string()));
            }
        }
    }

    Err(Error::Generic("Max retries exceeded".to_string()))
}