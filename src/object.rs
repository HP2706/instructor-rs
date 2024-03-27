use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidateArgs, ValidationErrors};// Import serde_json Error
use std::result::Result::Err; // Import std Result
use schemars::JsonSchema;
use crate::traits::LoadFromJson;
use serde_json::Error as SerdeJsonError; 

#[derive(Debug, Default, Validate, Deserialize, Serialize, JsonSchema)]
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

impl LoadFromJson<TestStruct> for TestStruct {
    fn load_from_json(json: &str) -> Result<TestStruct, SerdeJsonError> {
        serde_json::from_str::<TestStruct>(json)
    }
}

impl Validate for TestStruct {
    fn validate(&self) -> Result<(), ValidationErrors> {
        //this is a workaround, fix later
        self.validate()
    }
}


