use schemars::JsonSchema;
use std::clone::Clone;
use std::marker::{Send, Sync};
use instructor_rs::enums::InstructorResponse;
use instructor_rs::error::Error as InstructorError;


fn test_func<T>() -> bool 
where T: Send + Sync
{true}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn check_send_sync_impl() {
        #[derive(
            JsonSchema, serde::Serialize, Debug, Default, 
            validator::Validate, serde::Deserialize, Clone
        )]
        struct Number {
            //value: description
            #[schemars(description = "An integer value")]
            value1: i64,
        }     
        test_func::<InstructorError>();
        test_func::<Number>();
        //test_func::<InstructorResponse<Number>>();

    }
}

