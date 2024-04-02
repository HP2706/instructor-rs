use instructor_rs::traits::OpenAISchema;
use schemars::JsonSchema;
use validator::ValidationError;
use std::clone::Clone;
use instructor_rs::enums::IterableOrSingle;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn model_validate_json_no_args() {
        #[derive(
            JsonSchema, serde::Serialize, Debug, Default, 
            validator::Validate, serde::Deserialize, Clone, Copy
        )]
        struct Number {
            //value: description
            #[schemars(description = "An integer value")]
            value1: i64,
    
        }       
        let text = "{\"value1\": 10},\n{\"value1\": -5},\n{\"value1\": 1000},\n{\"value1\": -789},\n{\"value1\": 0},\n{\"value1\": 999999},\n{\"value1\": -123456},\n{\"value1\": 42},\n{\"value1\": -9876},\n{\"value1\": 1}\n";

        let out = Number::model_validate_json(
            &IterableOrSingle::Iterable(Number::default()), 
            text, 
            &()
        );

        //pass if Ok comes back fail if Err
        match out {
            Ok(_) => assert_eq!(true, true),
            Err(_) => {
                println!("Error: {:?}", out);
                assert_eq!(true, false)},
        }


    }


    #[test]
    pub fn model_validate_json_with_args() {
        #[derive(
            JsonSchema, serde::Serialize, Debug, Default, 
            validator::Validate, serde::Deserialize, Clone, Copy
        )]
        struct Number {
            //value: description
            #[schemars(description = "An integer value")]
            #[validate(custom(function = "validate", arg = "i64"))]
            value1: i64,
        }       

        fn validate(value: i64, arg: i64) -> Result<(), ValidationError> {
            if value < arg {
                let err_message = "value1 must be greater than arg";
                return Err(ValidationError::new(err_message));
            }
            Ok(())
        }

        let text = "{\"value1\": 10},\n{\"value1\": 1},\n{\"value1\": 1000},\n{\"value1\": 789},\n{\"value1\": 10},\n{\"value1\": 999999},\n{\"value1\": 123456},\n{\"value1\": 42},\n{\"value1\": 9876},\n{\"value1\": 2}\n";
        let out1 = Number::model_validate_json(
            &IterableOrSingle::Iterable(Number::default()), 
            text, 
            &1
        );

        //pass if Ok comes back fail if Err
        match out1 {
            Ok(_) => assert_eq!(true, true),
            Err(_) => {
                println!("Error: {:?}", out1);
                assert_eq!(true, false)},
        }

        
        let out2 = Number::model_validate_json(
            &IterableOrSingle::Iterable(Number::default()), 
            text, 
            &10
        );
        println!("out2: {:?}", out2);
        match out2 {
            Ok(_) => {
                println!("Error: {:?}", out2);
                assert_eq!(false, true)
            },
            Err(_) => {
                assert_eq!(false, false)},
        }

    }
}

