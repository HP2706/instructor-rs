use crate::mode::Mode;
use crate::error::Error;
use crate::process_response::process_response_async;
use crate::openai_schema::{BaseSchema, BaseArg};
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


/// this function generates the retry messages for the given mode and exception, 
/// to better inform the llm as to how to fix the error
/// # Arguments
/// * `model_message`: `String` - the model message to use for the retry
/// * `mode`: `Mode` - the mode to use for processing the response
/// * `exception`: `impl fmt::Display` - the exception to use for the retry
/// # Returns
/// * `Vec<ChatCompletionRequestMessage>` - the retry messages
pub fn reask_messages(
    model_message: String,
    mode: Mode,
    exception: impl fmt::Display,
) -> Vec<ChatCompletionRequestMessage> {
    

    //we extract the message from the stream or simply via message.choices[0].message.content
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

///This function takes a reference to a function as input.
///arguable whether this is a good idea, but trying to replicate the instructpr api as closely as possible
/// the it tries to process the response, if suceeding it returns the response else, it tri until it reaches max_retries
/// #Arguments 
/// * `func` a function that takes CreateChatCompletionRequest and returns a future of type Result<ChatCompletionResponseWrapper, OpenAIError>
/// * `response_model` the response model to use for processing the response
/// * `validation_context` the validation context to use for validating each struct
/// * `kwargs` the request object to modify
/// * `max_retries` the maximum number of retries to attempt
/// * `mode` the mode to use for processing the response 
pub async fn retry_async<T, A>(
    func: Box<dyn Fn(CreateChatCompletionRequest) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponseWrapper, OpenAIError>> + Send>> + Send + 'static>,
    response_model: IterableOrSingle<T>,
    validation_context: A,
    kwargs: &mut CreateChatCompletionRequest,
    max_retries: usize,
    mode: Mode,
) -> Result<InstructorResponse<T>, Error>
where
    T: ValidateArgs<'static, Args = A> + BaseSchema + 'static,
    A: BaseArg,
{
    let mut attempt = 0;

    while attempt < max_retries {
        let response = func(kwargs.clone());
        match response.await {
            Ok(_response) => {
                //we fetch the model message from the response before we process the response
                let model_message = _response.get_llm_test_response(mode);
                let result = process_response_async(
                    _response,
                    response_model.clone(),
                    &validation_context,
                    mode,
                ).await;

                match result {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        //TODO think about how would 
                        //can use response here and whether you can use it as is or not
                        if kwargs.stream.unwrap_or(false) {
                            return Err(e);
                        }
                        
                        match model_message {
                            Some(message) => {
                                let messages = reask_messages(message, mode, e);
                                kwargs.messages.extend(messages);
                                attempt += 1;
                                continue;
                            }
                            None => {
                                return Err(e);
                            }
                        }
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