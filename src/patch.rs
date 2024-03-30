
use openai_api_rs::v1::error::APIError;
use crate::process_response::{process_response, handle_response_model};
use crate::enums::{Iterable, IterableOrSingle};
use crate::retry::retry_sync;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{ChatCompletionRequest, ChatCompletionResponse};


use serde::{Deserialize, Serialize};
use validator::Validate;// Import serde_json Error
use schemars::JsonSchema;
use crate::traits::OpenAISchema;
use validator::ValidateArgs;
use crate::mode::Mode;
use crate::enums::Error;
use crate::enums::InstructorResponse;

#[derive(Debug)]
pub struct InstructorChatCompletionCreate<T>
where
    T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchema<T> + schemars::JsonSchema,
{
    pub kwargs: ChatCompletionRequest,
    pub response_model: Option<IterableOrSingle<T>>,
    pub validation_context: Option<T::Args>,
    pub max_retries: usize,
    pub mode: Mode,
}

// Define a wrapper type for the Client.
pub struct Patch {
    client: Client,
    mode: Option<Mode>,
}

impl Patch {
    pub fn chat_completion<T>(
        &mut self, 
        mut create: InstructorChatCompletionCreate<T>
    ) -> Result<InstructorResponse<T>, Error>

    where T: ValidateArgs<'static> + Default + Serialize + for<'de> Deserialize<'de> + OpenAISchema<T> + schemars::JsonSchema
    {
        // if no mode is provided, default to Mode::JSON
        match self.mode {
            Some(mode) => {},
            None => self.mode = Some(Mode::JSON),
        }

        let (response_model, mut kwargs) = handle_response_model(
            Some(create.response_model.unwrap()), 
            self.mode.unwrap(), 
            create.kwargs
        ).map_err(|e| e)?;


        let func = Box::new(|kwargs| {
            self.client.chat_completion(kwargs)
        });

        return retry_sync(
            func,
            response_model,
            create.validation_context,
            &mut kwargs,
            create.max_retries,
            create.mode
        );
    }
}








