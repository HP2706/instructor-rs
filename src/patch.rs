
use crate::process_response::handle_response_model;
use crate::iterable::IterableOrSingle;
use crate::retry::retry_sync;
use async_openai::types::{CreateChatCompletionRequestArgs, CreateChatCompletionRequest};
use async_openai::Client;
use async_openai::error::OpenAIError;
use async_openai::config::Config;

use crate::traits::BaseSchema;
use validator::ValidateArgs;
use crate::mode::Mode;
use crate::error::Error;
use crate::enums::{InstructorResponse, ChatCompletionResponseWrapper};


// Define a wrapper type for the Client.
pub struct Patch<C : Config> {
    pub client: Client<C>,
    pub mode: Option<Mode>,
}

impl<C : Config> Patch<C> {
    pub fn chat_completion<T, A>(
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
        

        let func : Box<dyn Fn(CreateChatCompletionRequest) -> Result<ChatCompletionResponseWrapper, OpenAIError>> = Box::new(|kwargs| {
            match kwargs.stream {
                Some(false) | None => {
                    match self.client.chat().create(kwargs) {
                        Ok(res) => Ok(ChatCompletionResponseWrapper::Single(res)),
                        Err(e) => Err(e),
                    }
                },
                Some(true) => {
                    self.client.chat().create_stream(kwargs)
                }
            }
            
        });

        return retry_sync(
            func,
            response_model,
            validation_context,
            &mut kwargs,
            max_retries,
            stream,
            self.mode.unwrap(),
        );
    }
    
}








