use schemars::JsonSchema;
use validator::ValidateArgs;
use serde::{Deserialize, Serialize};
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::enums::Error;
use crate::mode::Mode;
use crate::utils::extract_json_from_codeblock;
pub trait OpenAISchema<Args, T> {
    type Args;
    fn openai_schema() -> String;

    fn model_validate_json(data: &str, validation_context: &Args) -> Result<Self, Error>
    where
        Self: Sized + ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>;
    
    fn from_response(
        response: &ChatCompletionResponse,
        validation_context: &Args,
        mode: Mode,
    ) -> Result<Self, Error>
    where
        Self: Sized + ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>;
    
    fn parse_json(
        completion: &ChatCompletionResponse,
        validation_context: &Args,
    ) -> Result<Self, Error>
    where
        Self: Sized + ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>;
}

impl<A, T> OpenAISchema<A, T> for T
where
    T: ValidateArgs<'static, Args=A> + Serialize + for<'de> Deserialize<'de> + JsonSchema,
    A: 'static + Copy,
{
    type Args = A;

    // The rest of your implementation remains the same...

    fn openai_schema() -> String {
        let schema = schemars::schema_for!(T);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    fn model_validate_json(data: &str, validation_context: &Self::Args) -> Result<Self, Error>
    where
        Self: Sized + ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>,
    {
        match serde_json::from_str::<T>(data) {
            Ok(data) => match data.validate_args(*validation_context) {
                Ok(_) => Ok(data),
                Err(e) => Err(Error::ValidationErrors(e)),
            },
            Err(e) => Err(Error::SerdeError(e)),
        }
    }

    fn from_response(
        response: &ChatCompletionResponse,
        validation_context: &Self::Args,
        mode: Mode,
    ) -> Result<Self, Error>
    where
        Self: Sized + ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>,
    {
        match mode {
            Mode::JSON | Mode::JSON_SCHEMA | Mode::MD_JSON => {
                Self::parse_json(response, validation_context)
            }
            _ => Err(Error::NotImplementedError(
                "This feature is not yet implemented.".to_string(),
            )),
        }
    }

    fn parse_json(
        completion: &ChatCompletionResponse,
        validation_context: &Self::Args,
    ) -> Result<Self, Error>
    where
        Self: Sized + ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>,
    {
        let text = completion.choices[0].message.content.clone().unwrap();
        let json_extract = extract_json_from_codeblock(&text);
        Self::model_validate_json(&json_extract, validation_context)
    }
}

