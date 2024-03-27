use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidateArgs};// Import serde_json Error
use std::result::Result::Err; // Import std Result
use schemars::JsonSchema;
use crate::types::JsonError;

pub trait JsonLoad {
    fn load_from_json(json: &str) -> Result<Self, JsonError>
    where
        Self: Sized + Validate;
}

#[derive(Debug, Validate, Deserialize, Serialize, JsonSchema)]
pub struct SignupData {
    #[validate
        (custom(function = "validate_value", arg = "&'v_a str", message = "Value is not contained within arg"))
    ]
    pub value: String,
}

fn validate_value(sentence: &str, arg : &str) -> Result<(), ValidationError> {
    if !(arg.contains(sentence)) {
        let err_message = format!("Value {} is not contained within arg: {}\n", sentence, arg);
        let leaked_message = Box::leak(err_message.into_boxed_str());
        return Err(ValidationError::new(leaked_message));
    }

    Ok(())
}

impl JsonLoad for SignupData {
    fn load_from_json(json: &str) -> Result<Self, JsonError> {
        let data = serde_json::from_str::<SignupData>(json).
            map_err(JsonError::SerdeJson)?;
        let mut text = "hello from the other side";
        match data.validate_args(&text) {
            Ok(_) => Ok(data),
            Err(e) => Err(JsonError::Validation(e)),
        }
    }
}

impl Default for SignupData {
    fn default() -> Self {
        SignupData {
            value: "".to_string(),
        }
    }
}