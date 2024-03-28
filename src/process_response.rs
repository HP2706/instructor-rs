use validator::{ValidateArgs, ValidationErrors};
use crate::mode::Mode;
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use serde::{Serialize, Deserialize};


pub fn process_response<T>(
    response: &ChatCompletionResponse,
    response_model: T,
    stream: bool,
    validation_context: T::Args,
    strict: Option<bool>,
    mode: Mode,
) -> Result<T, ValidationErrors>
where
    T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>,
{
    let text = response.choices[0].message.content.clone().unwrap();
    deserialize_and_validate(text, validation_context)
}

fn deserialize_and_validate<T>(
    text: String,
    validation_context: T::Args,
) -> Result<T, ValidationErrors>
where
    T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>,
{
    match serde_json::from_str::<T>(&text) {
        Ok(data) => match data.validate_args(validation_context) {
            Ok(_) => Ok(data),
            Err(e) => Err(e),
        },
        Err(_) => panic!("Failed to deserialize response"),
    }
}