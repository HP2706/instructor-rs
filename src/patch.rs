
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


// Define a wrapper type for the Client.
pub struct Patch {
    pub client: Client,
    pub mode: Option<Mode>,
}

impl Patch {
    pub fn chat_completion<T>(
        &mut self, 
        response_model: Option<IterableOrSingle<T>>,
        validation_context: Option<<T as OpenAISchema<T>>::Args>,
        max_retries: usize,
        kwargs: ChatCompletionRequest
    ) -> Result<InstructorResponse<T>, Error>

    where T: ValidateArgs<'static> + Default + Serialize + for<'de> Deserialize<'de> + OpenAISchema<T, Args = T> + schemars::JsonSchema
    {
        // if no mode is provided, default to Mode::JSON
        match self.mode {
            Some(mode) => {},
            None => self.mode = Some(Mode::JSON),
        }

        let (response_model, mut kwargs) = handle_response_model(
            Some(response_model.unwrap()), 
            self.mode.unwrap(), 
            kwargs
        ).map_err(|e| e)?;

        let func = Box::new(|kwargs| {
            self.client.chat_completion(kwargs)
        });

        return retry_sync(
            func,
            response_model,
            validation_context,
            &mut kwargs,
            max_retries,
            self.mode.unwrap(),
        );
    }
}








