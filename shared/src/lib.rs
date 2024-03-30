pub mod mode;
pub mod enums;
pub mod utils;

use crate::utils::extract_json_from_codeblock;
use crate::mode::Mode;
use validator::{ValidateArgs, ValidationErrors};
use serde::{Deserialize, Serialize};
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::enums::Error;
pub trait OpenAISchemaSpec {
    fn openai_schema() -> String;

    fn model_validate_json<T>(
        data: &str,
        validation_context: T::Args,
    ) -> Result<T, Error>
    where
        T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>;
    

    fn from_response<T>(
        response: &ChatCompletionResponse,
        validation_context: T::Args,
        mode: Mode,
    ) -> Result<T, Error>
    where
        T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchemaSpec;

    fn parse_json<T>(
        completion : &ChatCompletionResponse,
        validation_context: T::Args,
    ) -> Result<T, Error>
    where
        T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchemaSpec;


    //TODOS

    //fn parse_tools
    //fn parse_functions

}

