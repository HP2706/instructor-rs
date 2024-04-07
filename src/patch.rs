
use crate::process_response::handle_response_model;
use crate::enums::IterableOrSingle;
use crate::retry::retry_async;
use async_openai::types::{CreateChatCompletionRequestArgs, CreateChatCompletionRequest};
use std::marker::{Send, Sync};
use async_openai::Client;
use async_openai::error::OpenAIError;
use async_openai::config::Config;
use crate::openai_schema::{BaseSchema, BaseArg};
use validator::ValidateArgs;
use crate::mode::Mode;
use crate::error::Error;
use crate::enums::{InstructorResponse, ChatCompletionResponseWrapper};
use std::pin::Pin;
use std::future::{Future, ready};
// Define a wrapper type for the Client.

#[derive(Debug, Clone)]
pub struct Patch<C : Config> {
    pub client: Client<C>,
    pub mode: Option<Mode>,
}


impl<C> Patch<C> 
where
    C: Config + Clone + Send + Sync + 'static,
{
    /// Initiates a chat completion request with the OpenAI API.
    /// 
    /// # Arguments
    /// 
    /// * `response_model`: `IterableOrSingle<T>` - Determines the type of response model to use. 
    ///   Can be either a single instance or an iterable collection of instances, depending on the use case.
    /// * `validation_context`: `A` - The context or data used for validating the request. 
    ///   The type `A` must implement the `BaseArg` trait.
    /// * `max_retries`: `usize` - The maximum number of retries for the request in case of failures.
    /// * `stream`: `bool` - A flag indicating whether the response should be streamed. 
    ///   If `true`, the response is streamed; otherwise, a single response is returned.
    /// * `kwargs`: `CreateChatCompletionRequest` - Additional keyword arguments to customize the chat completion request.
    /// 
    /// # Returns
    /// 
    /// A `Result` type that, on success, contains an `InstructorResponse<T>`, 
    /// which wraps the response model(s) in the specified format (either single or iterable). 
    /// On failure, it returns an `Error`.
    pub async fn chat_completion<T, A>(
        &self, 
        response_model:IterableOrSingle<T>,
        validation_context: A,
        max_retries: usize,
        kwargs: CreateChatCompletionRequest
    ) -> Result<InstructorResponse<T>, Error>
    where
        T: ValidateArgs<'static, Args=A> + BaseSchema + 'static,
        A: BaseArg,
    {

        let mut kwargs = kwargs.clone();
        // if no mode is provided, default to Mode::JSON
        let mode = match self.mode {
            Some(mode) => mode,
            None => Mode::JSON,
        };

        let response_model = handle_response_model(
            response_model, 
            mode, 
            &mut kwargs
        ).map_err(|e| e)?;
        
        let client = self.client.clone();
        let func: Box<
            dyn Fn(CreateChatCompletionRequest) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponseWrapper, OpenAIError>> + Send>>
                + Send
                + Sync,
        > = Box::new(move |kwargs| {
            let client = client.clone();
            Box::pin(async move {
                match kwargs.stream {
                    Some(false) | None => {
                        let result = client.chat().create(kwargs).await;
                        match result {
                            Ok(res) => Ok(ChatCompletionResponseWrapper::Single(res)),
                            Err(e) => Err(e),
                        }
                    }
                    Some(true) => {
                        let result = client.chat().create_stream(kwargs).await;
                        match result {
                            Ok(res) => Ok(ChatCompletionResponseWrapper::Stream(res)),
                            Err(e) => Err(e),
                        }
                    }
                }
            })
        });

        retry_async(
            func,
            response_model,
            validation_context,
            &mut kwargs,
            max_retries,
            self.mode.unwrap(),
        ).await
    }
    
}








