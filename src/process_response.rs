use validator::{ValidateArgs, ValidationErrors};
use crate::mode::Mode;
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::utils::Load_And_Validate;
use serde::{Serialize, Deserialize};

pub fn get_text(response: ChatCompletionResponse) -> String {
    let choices = &response.choices;
    choices[0].message.content.clone().unwrap()
}

pub fn process_response<'v_a, T>(
    response: ChatCompletionResponse,
    response_model: T,
    stream: bool,
    validation_context: T::Args,
    strict: Option<bool>,
    mode: Mode,
) -> Result<T, ValidationErrors>
where
    T: ValidateArgs<'v_a> + Serialize + Deserialize<'v_a>,
{
    let text = response.choices[0].message.content.clone().unwrap();
    Load_And_Validate::<'v_a, T>(&text, validation_context)
}
