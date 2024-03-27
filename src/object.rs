use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidateArgs, ValidationErrors};// Import serde_json Error
use std::result::Result::Err; // Import std Result
use schemars::JsonSchema;
use crate::types::JsonError;

pub trait JsonLoad {
    fn load_from_json(json: &str) -> Result<Self, JsonError>
    where
        Self: Sized,
        for<'a> &'a Self: Validate;
}

#[derive(Debug, Validate, Deserialize, Serialize, JsonSchema)]
pub struct TestStruct {
    #[validate(custom(function = "validate_value", arg = "(i64, i64)"))]
    pub value: i64,
}

fn validate_value(v: i64, arg : (i64, i64)) -> Result<(), ValidationError> {
    if v != arg.0*arg.1 {
        return Err(ValidationError::new("value must be equal to the product of the two arguments"));
    } 
    Ok(())
}

impl JsonLoad for TestStruct {
    fn load_from_json(json: &str) -> Result<Self, JsonError> {
        let data = serde_json::from_str::<TestStruct>(json).
            map_err(JsonError::SerdeJson)?;
        let mut text = "hello from the other side";

        match data.validate_args((7, 10)) {
            Ok(_) => Ok(data),
            Err(e) => Err(JsonError::Validation(e)),
        }
    }
}

impl Validate for TestStruct {
    fn validate(&self) -> Result<(), ValidationErrors> {
        // Implement validation logic here.
        // For simplicity, we're directly calling validate on the instance,
        // which is possible because TestStruct itself derives Validate.
        self.validate()
    }
}


impl Default for TestStruct {
    fn default() -> Self {
        TestStruct {
            value: 70,
        }
    }
}