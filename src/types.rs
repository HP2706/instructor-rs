use validator::ValidationErrors;
use std::fmt;
use std::error::Error as StdError;
use futures::stream::Stream;
use std::pin::Pin;
use crate::error::Error;
pub type JsonStream = Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>;


#[derive(Debug)]
pub enum JsonError {
    Validation(ValidationErrors),
    Generic(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsonError::Validation(ref err) => write!(f, "Validation error: {}", err),
            JsonError::Generic(ref err) => write!(f, "Error: {}", err),
        }
    }
}

impl From<ValidationErrors> for JsonError {
    fn from(err: ValidationErrors) -> JsonError {
        JsonError::Validation(err)
    }
}


#[derive(Debug)]
pub struct RetryError {
    pub last_attempt: Box<dyn StdError>, // Simplified for example purposes
}

