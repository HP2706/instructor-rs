use instructor_rs::openai_schema::OpenAISchema;
use schemars::JsonSchema;
use validator::ValidationError;
use std::clone::Clone;
use instructor_rs::enums::IterableOrSingle;
use instructor_rs::utils::{
    extract_json_from_codeblock, 
    create_chat_completion_response, create_tool_call
};
use instructor_rs::error::Error;
use async_openai::types::CreateChatCompletionResponse;

use validator::Validate;
use serde::{Deserialize, Serialize};
use model_traits_macro::derive_all;



#[derive_all]
struct TestStruct {
    value1: i64,
}     

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn model_validate_json_no_args() {
        #[derive_all]
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
        #[derive_all]
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

    #[test]
    pub fn parse_json(){
        let positive_example = "{\"value1\": 10}";
        let positive_out = TestStruct::model_validate_json(
            &IterableOrSingle::Iterable(TestStruct::default()), 
            positive_example, 
            &()
        );
        match positive_out {
            Ok(_) => assert_eq!(true, true),
            Err(_) => {
                println!("Error: {:?}", positive_out);
                assert_eq!(true, false)},
        }

        let negative_example = "{\"value1\": \"hello\"}";
        let negative_out = TestStruct::model_validate_json(
            &IterableOrSingle::Iterable(TestStruct::default()), 
            negative_example, 
            &()
        );
        match negative_out {
            Ok(_) => {
                println!("Error: {:?}", negative_out);
                assert_eq!(false, true)
            },
            Err(_) => assert_eq!(true, true),
        }
    }

    #[test]
    pub fn test_extract_json_from_codeblock(){
        // Positive test case
        let well_formed_codeblock = "```json\n{\"value1\": 10}\n```";
        match extract_json_from_codeblock(well_formed_codeblock) {
            Ok(json) => assert_eq!(json, "{\"value1\": 10}"),
            Err(_) => panic!("Expected valid JSON extraction"),
        }

        // Negative test case
        let misformed_codeblock = "```json\nvalue1: 10\n```";
        match extract_json_from_codeblock(misformed_codeblock) {
            Ok(_) => panic!("Expected an error due to invalid JSON format"),
            Err(e) => match e {
                Error::JsonExtractionError(msg) => assert_eq!(msg, "No JSON found".to_string()),
                _ => panic!("Unexpected error type"),
            },
        }

        let wrongly_nested_codeblock = "```json\n{\"value1\": 10\n```";
        match extract_json_from_codeblock(wrongly_nested_codeblock) {
            Ok(_) => panic!("Expected an error due to invalid JSON format"),
            Err(e) => match e {
                Error::JsonExtractionError(msg) => assert_eq!(msg, "No JSON found".to_string()),
                _ => panic!("Unexpected error type"),
            },
        }
    }

    #[test]
    pub fn test_parse_tools_iterable_negative(){
        let wrong_tool = create_tool_call("wrong_tool".to_string(), "{}".to_string());
        let right_tool = create_tool_call("TestStruct".to_string(), "{\"value1\": 10}".to_string());
        let response = create_chat_completion_response(
            Some(vec![wrong_tool, right_tool]), None
        );

        
        let res = TestStruct::parse_tools(
            &IterableOrSingle::Iterable(TestStruct::default()), 
            &response, 
            &()
        );
        match res {
            Ok(_) => assert_eq!(false, true),
            Err(_) => assert_eq!(true, true),
        }
    }

    #[test]
    pub fn test_parse_tools_iterable_positive(){
        let right_tool = create_tool_call("TestStruct".to_string(), "{\"value1\": 10}".to_string());
        let response = create_chat_completion_response(
            Some(vec![right_tool; 2]), None
        );

        let res = TestStruct::parse_tools(
            &IterableOrSingle::Iterable(TestStruct::default()), 
            &response, 
            &()
        );
        match res {
            Ok(_) => assert_eq!(true, true),
            Err(_) => assert_eq!(false, true),
        }
    }

    #[test]
    pub fn test_parse_tools_negative(){
        let wrong_tool = create_tool_call("wrong_tool".to_string(), "{}".to_string());
        let response = create_chat_completion_response(Some(vec![wrong_tool]), None);
        
        let res = TestStruct::parse_tools(
            &IterableOrSingle::Single(TestStruct::default()), 
            &response, 
            &()
        );
        match res {
            Ok(_) => { 
            println!("Error: {:?}", res);
            assert_eq!(false, true)
            },
            Err(_) => assert_eq!(true, true),
        }
    }

    #[test]
    pub fn test_parse_tools_positive(){
        #[derive_all]
        struct MyFunc {
            value: i64,
            value2: String,
        }

        let positive_tool = create_tool_call("MyFunc".to_string(), "{\"value\": 10, \"value2\": \"hello\"}".to_string());
        let response = create_chat_completion_response(Some(vec![positive_tool]), None);
        let res = MyFunc::parse_tools(
            &IterableOrSingle::Single(MyFunc::default()), 
            &response, 
            &()
        );

        match res {
            Ok(_) => assert_eq!(true, true),
            Err(_) => assert_eq!(false, true),
        }
    }

    #[test]
    pub fn test_parse_tools_positive_validation_positive(){
        #[derive_all]
        struct MyFunc {
            value: i64,

            #[validate(custom(function = "validate", arg = "i64"))]
            value2: String,
        }

        fn validate(value2: &str, arg: i64) -> Result<(), ValidationError> {
            if value2.len() < arg as usize {
                let err_message = "value2 must be more than arg";
                return Err(ValidationError::new(err_message));
            }
            Ok(())
        }

        let positive_tool = create_tool_call("MyFunc".to_string(), "{\"value\": 10, \"value2\": \"to be or not to be, that is the question\"}".to_string());
        let response = create_chat_completion_response(Some(vec![positive_tool]), None);
        let res = MyFunc::parse_tools(
            &IterableOrSingle::Single(MyFunc::default()), 
            &response, 
            &(10)
        );

        println!("res: {:?}", res);

        match res {
            Ok(_) => assert_eq!(true, true),
            Err(_) => assert_eq!(false, true),
        }
    }

    #[test]
    pub fn test_parse_tools_positive_validation_negative(){
        #[derive_all]
        struct MyFunc {
            value: i64,

            #[validate(custom(function = "validate", arg = "i64"))]
            value2: String,
        }

        fn validate(value2: &str, arg: i64) -> Result<(), ValidationError> {
            if value2.len() < arg as usize {
                let err_message = "value2 must be more than arg";
                return Err(ValidationError::new(err_message));
            }
            Ok(())
        }

        let positive_tool = create_tool_call("MyFunc".to_string(), "{\"value\": 10, \"value2\": \"hi\"}".to_string());
        let response = create_chat_completion_response(Some(vec![positive_tool]), None);
        let res = MyFunc::parse_tools(
            &IterableOrSingle::Single(MyFunc::default()), 
            &response, 
            &(10)
        );

        match res {
            Ok(_) => assert_eq!(false, true),
            Err(_) => assert_eq!(true, true),
        }
    }
}

