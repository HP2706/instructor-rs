use std::error::Error;

// Assuming ValidationError, JSONDecodeError, and RetryError are defined somewhere
// and implement std::error::Error

//NotImplementedError

pub struct NotImplementedError {
    pub message: String,
}