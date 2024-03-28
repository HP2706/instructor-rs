use crate::mode::Mode;
use crate::process_response::process_response;
use crate::types::{JsonError, RetryError};
use validator::ValidateArgs;
use openai_api_rs::v1::chat_completion::{ChatCompletionRequest, ChatCompletionResponse, ChatCompletionMessage, MessageRole, Content};
use std::fmt;



pub fn reask_messages(
    response: &ChatCompletionResponse, mode: Mode, exception: &impl fmt::Display
) -> impl Iterator<Item = ChatCompletionMessage> {
    let mut messages: Vec<ChatCompletionMessage> = Vec::new();

    match mode {
        /* Mode::ANTHROPIC_TOOLS => { //TODO when anthropic tools is implemented
            messages.push(
                ChatCompletionMessage {
                    role: MessageRole::user,
                    content: Content::Text(format!(
                        "Validation Error found:\n{}\nRecall the function correctly, fix the errors",
                        exception
                    )),
                    name: None,
                }
            );
        } */
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


pub fn retry_sync<'v_a, T>(
    func: Box<dyn Fn(ChatCompletionRequest) -> Result<T, JsonError>>,
    response: ChatCompletionResponse,
    response_model: T,
    args : ChatCompletionRequest, 
    stream: bool,
    validation_context: T::Args,
    strict: Option<bool>,
    mode: Mode,
    max_retries: usize
) -> T
where
    T: ValidateArgs<'v_a>,
{
    let mut attempt = 0;
    let mut messages : Vec<ChatCompletionMessage> = Vec::new();

    loop {
        //TODO
        attempt += 1;
    }
}
