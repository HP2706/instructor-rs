use crate::mode::Mode;
use crate::error::Error;
use crate::process_response::process_response;
use crate::traits::BaseSchema;
use validator::ValidateArgs;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionMessage, MessageRole, Content
};
use std::fmt;
use openai_api_rs::v1::error::APIError;
use crate::enums::InstructorResponse;
use crate::iterable::IterableOrSingle;

pub fn reask_messages(
    model_message: String,
    mode: Mode,
    exception: impl fmt::Display,
) -> Vec<ChatCompletionMessage> {
    

    //we extract the message from the stream or simply via message.choices[0].message.content
    //let message_content = response.get_message();
    let message = ChatCompletionMessage {
        role: MessageRole::assistant,
        content: Content::Text(model_message),
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
    func: Box<dyn Fn(ChatCompletionRequest) -> Result<ChatCompletionResponseWrapper, APIError> + 'f>,
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
                let model_message = _response.get_message();
                let result = process_response(
                    _response,
                    &response_model,
                    stream,
                    &validation_context,
                    mode,
                );

                match result {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        println!("Error: {}", e);
                        //TODO think about how would 
                        //can use response here and whether you can use it as is or not
                        match model_message {
                            Some(message) => {
                                let messages = reask_messages(message, mode, e);
                                println!("number of messages before: {}", kwargs.messages.len());
                                kwargs.messages.extend(messages);
                                println!("number of messages after: {}", kwargs.messages.len());
                                attempt += 1;
                                continue;
                            }
                            None => {
                                return Err(Error::Generic(format!("Error: {}", e).to_string()));
                            }
                        }
                    } //TODO BETTER ERROR HANDLING ANOTHER LOOP SHOULD ONLY GET RUN IF ERROR IS 
                    //JSONDECODEERROR(SERDE ERROR) OR
                    //VALIDATIONERROR
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