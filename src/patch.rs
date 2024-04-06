
use crate::process_response::handle_response_model;
use crate::iterable::IterableOrSingle;
use crate::retry::retry_async;
use async_openai::types::{CreateChatCompletionRequestArgs, CreateChatCompletionRequest};
use std::marker::{Send, Sync};
use async_openai::Client;
use async_openai::error::OpenAIError;
use async_openai::config::Config;
use crate::traits::BaseSchema;
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
    pub async fn chat_completion<T, A>(
        &self, 
        response_model:IterableOrSingle<T>,
        validation_context: A,
        max_retries: usize,
        stream: bool,
        kwargs: CreateChatCompletionRequest
    ) -> Result<InstructorResponse<A, T>, Error>
    where
        T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
        A: 'static + Copy,
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
                + Sync
                + 'static,
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
                        panic!("Not implemented yet");
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
            stream,
            self.mode.unwrap(),
        ).await
    }
    
}








