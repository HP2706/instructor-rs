use validator::ValidationErrors;
use std::fmt;
use std::error::Error;

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
    pub last_attempt: Box<dyn Error>, // Simplified for example purposes
}


struct OpenAiKwargs {
    

}