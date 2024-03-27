use validator::ValidationErrors;
use serde_json::Error as SerdeJsonError; 
use std::fmt;

#[derive(Debug)]
pub enum JsonError {
    SerdeJson(SerdeJsonError),
    Validation(ValidationErrors),
    Generic(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsonError::SerdeJson(ref err) => write!(f, "Serialization error: {}", err),
            JsonError::Validation(ref err) => write!(f, "Validation error: {}", err),
            JsonError::Generic(ref err) => write!(f, "Error: {}", err),
        }
    }
}

impl From<SerdeJsonError> for JsonError {
    fn from(err: SerdeJsonError) -> JsonError {
        JsonError::SerdeJson(err)
    }
}

impl From<ValidationErrors> for JsonError {
    fn from(err: ValidationErrors) -> JsonError {
        JsonError::Validation(err)
    }
}
