use crate::mode::Mode;
use crate::error::Error;
use crate::process_response::process_response_async;
use crate::traits::{BaseSchema, BaseArg};
use validator::ValidateArgs;
use std::fmt;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, 
    ChatCompletionRequestAssistantMessage, Role, CreateChatCompletionRequest
};
use std::pin::Pin;
use std::future::Future;
use async_openai::error::OpenAIError;
use crate::enums::{InstructorResponse, ChatCompletionResponseWrapper};
use crate::enums::IterableOrSingle;

pub fn reask_messages(
    model_message: String,
    mode: Mode,
    exception: impl fmt::Display,
) -> Vec<ChatCompletionRequestMessage> {
    

    //we extract the message from the stream or simply via message.choices[0].message.content
    //let message_content = response.get_message();
    let message = ChatCompletionRequestMessage::Assistant(
        ChatCompletionRequestAssistantMessage{
            role: Role::Assistant,
            content: Some(model_message),
            name: None,
            tool_calls : None,
            function_call: None,
        }
    );
       
    let mut messages: Vec<ChatCompletionRequestMessage> = vec![message];

    match mode {
        Mode::MD_JSON => {
            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage{
                    role: Role::User,
                    content: ChatCompletionRequestUserMessageContent::Text(format!(
                        "Correct your JSON ONLY RESPONSE, based on the following errors:\n{}\n",
                        exception
                    )),
                    name: None,
                }   
            ));
        }
        _ => {
            messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage{
                role: Role::User,
                content: ChatCompletionRequestUserMessageContent::Text(format!(
                    "Recall the function correctly, fix the errors, exceptions found\n{}",
                    exception
                )),
                    name: None,
                }    
            ));
        }
    }

    messages
}

pub async fn retry_async<'f, T, A>(
    func: Box<dyn Fn(CreateChatCompletionRequest) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponseWrapper, OpenAIError>> + Send>> + Send + 'static>,
    response_model: IterableOrSingle<T>,
    validation_context: A,
    kwargs: &mut CreateChatCompletionRequest,
    max_retries: usize,
    stream: bool,
    mode: Mode,
) -> Result<InstructorResponse<A, T>, Error>
where
    T: ValidateArgs<'static, Args = A> + BaseSchema,
    A: BaseArg,
{
    let mut attempt = 0;

    while attempt < max_retries {
        println!("message to model\n\n {:?}", kwargs.messages);
        println!("attempt: {}", attempt);
        let response = func(kwargs.clone());
        match response.await {
            Ok(_response) => {
                let model_message = _response.get_message();
                let result = process_response_async(
                    &_response,
                    &response_model,
                    stream,
                    &validation_context,
                    mode,
                ).await;

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