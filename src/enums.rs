
use serde::{Deserialize, Serialize};
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::traits::BaseSchema;
use validator::{ValidateArgs, ValidationErrors};
use schemars::JsonSchema;
use std::fmt;
use serde_json::Error as SerdeError;


#[derive(Debug)]
pub enum Error {
    ValidationErrors(validator::ValidationErrors),
    ValidationError(validator::ValidationError),
    SerdeError(SerdeError),
    NotImplementedError(String),
    APIError(String),
    Generic(String),
    JsonExtractionError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ValidationErrors(ref err) => write!(f, "Validation error: {}", err),
            Error::ValidationError(ref err) => write!(f, "Validation error: {}", err),
            Error::SerdeError(ref err) => write!(f, "Serde error: {}", err),
            Error::NotImplementedError(ref err) => write!(f, "Not implemented: {}", err),
            Error::APIError(ref err) => write!(f, "API error: {}", err),
            Error::Generic(ref err) => write!(f, "Error: {}", err),
            Error::JsonExtractionError(ref err) => write!(f, "Error: {}", err),
        }
    }
}

//TODO implement more traits for the enum, for multiprocessing and ...


#[derive(Debug)]
pub enum InstructorResponse<A, T>
    where T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    One(T),
    Many(Vec<T>),
}


impl<T> IterableOrSingle<T>
where T: ValidateArgs<'static>
{
    // This method is now correctly placed outside the ValidateArgs trait impl block
    pub fn unwrap(self) -> T {
        match self {
            IterableOrSingle::Iterable(item) | IterableOrSingle::Single(item) => item,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, JsonSchema)]
pub enum IterableOrSingle<T>
where T: ValidateArgs<'static> 
{
    Iterable(T), 
    Single(T),
}

impl<'v_a, T> ValidateArgs<'static> for IterableOrSingle<T>
where
    T: ValidateArgs<'static>,
{
    type Args = T::Args;

    fn validate_args(&self, args: Self::Args) -> Result<(), ValidationErrors> {
        match self {
            IterableOrSingle::Iterable(item) | IterableOrSingle::Single(item) => {
                item.validate_args(args)
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Iterable<T> {
    VecWrapper(Vec<T>),
    // You can add more variants here if you need to wrap T in different iterable types
}


// Example usage
pub fn use_iterable_wrapper<T>(wrapper: Iterable<T>) {
    match wrapper {
        Iterable::VecWrapper(vec) => {
            for item in vec {
                //TODO Process each item
            }
        },
    }
}