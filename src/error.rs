use serde_json::Error as SerdeError;
use std::fmt;
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
