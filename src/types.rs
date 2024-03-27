use validator::ValidationErrors;
use std::fmt;

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
