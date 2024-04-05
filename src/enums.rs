use crate::traits::BaseSchema;
use validator::ValidateArgs;
use std::fmt;
use serde_json::Error as SerdeError;
use std::marker::{Send, Sync};
use crate::streaming::StreamingError;

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
    Stream(Box<dyn Iterator<Item = Result<T, StreamingError>>>),
}

pub enum MaybeStream<T> {
    Stream(Box<dyn Iterator<Item = Result<T, StreamingError>>>),
    One(T),
    Many(Vec<T>),
}

impl<T> MaybeStream<T> {
    ///gets the first item from the stream, or the first item in the vector, or the first item in the stream
    pub fn unwrap(self) -> Result<T, StreamingError> {
        match self {
            MaybeStream::One(item) => Ok(item),
            MaybeStream::Many(items) => Ok(items.into_iter().next().unwrap()),
            MaybeStream::Stream(iter) => iter.into_iter().next().unwrap(),
        }
    }
    //Stream(Box<dyn Iterator<Item = Result<T, StreamingError>>>),
}

pub enum MaybeStream<T> {
    //Stream(Box<dyn Iterator<Item = Result<T, StreamingError>>>),
    One(T),
    Many(Vec<T>),
}

/* impl<T> MaybeStream<T> {
    ///gets the first item from the stream, or the first item in the vector, or the first item in the stream
    pub fn unwrap(self) -> Result<T, StreamingError> {
        match self {
            MaybeStream::One(item) => Ok(item),
            MaybeStream::Many(items) => Ok(items.into_iter().next().unwrap()),
            MaybeStream::Stream(iter) => iter.into_iter().next().unwrap(),
        }
    }
}

impl<A, T> InstructorResponse<A, T>
where
    T: ValidateArgs<'static, Args = A> + BaseSchema<T>,
    A: 'static + Copy,
{
    pub fn unwrap(self) -> T {
        match self {
            InstructorResponse::One(item) => item,
            InstructorResponse::Many(mut items) => items.pop().expect("InstructorResponse::Many should not be empty"),
        }
    }
} */



