use crate::openai_schema::{BaseArg, BaseSchema};
use validator::{ValidateArgs, ValidationErrors};
use crate::error::Error;
use async_openai::types::{CreateChatCompletionResponse, ChatCompletionResponseStream};
use std::pin::Pin;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::fmt::{Formatter, Debug};


pub enum ChatCompletionResponseWrapper {
    Single(CreateChatCompletionResponse),
    Stream(ChatCompletionResponseStream),
}

impl ChatCompletionResponseWrapper {
    pub fn get_message(&self) -> Option<String> {
        match self {
            ChatCompletionResponseWrapper::Single(resp) => {
                let message = resp.choices.get(0).unwrap().message.content.clone().unwrap();
                Some(message)
            },
            ChatCompletionResponseWrapper::Stream(iter) => {
                /* let mut buffer = String::new();
                for result in iter {
                    match result {
                        Ok(ChatCompletionStreamingResponse::Chunk(chunk)) => {
                            match &chunk.choices[0].delta {
                                Delta::Content { content } => {
                                    buffer.push_str(&content);
                                }
                                Delta::Empty {} => {}
                            }
                        }
                        _ => {}
                    }
                };
                buffer */
                None
            }
        }
    }

    pub fn get_single(self) -> Result<CreateChatCompletionResponse, Error> {
        match self {
            ChatCompletionResponseWrapper::Single(resp) => Ok(resp),
            ChatCompletionResponseWrapper::Stream(iter) => Err(Error::Generic("Got a stream".to_string())),
        }
    }
}

//TODO implement more traits for the enum, for multiprocessing and ...
pub enum InstructorResponse<T>
    where T: ValidateArgs<'static> + BaseSchema,
{
    One(T),
    Many(Vec<T>),
    Stream(Pin<Box<dyn Stream<Item = Result<T, Error>> + Send>>),
}

pub enum MaybeStream<T> {
    Stream(Pin<Box<dyn Stream<Item = Result<T, Error>> + Send>>),
    One(T),
    Many(Vec<T>),
}

impl<T> InstructorResponse<T>
where
    T: ValidateArgs<'static> + BaseSchema,
{
    pub fn unwrap(self) -> Result<T, Error> {
        match self {
            InstructorResponse::One(item) => Ok(item),
            InstructorResponse::Many(mut items) => Ok(items.pop().expect("InstructorResponse::Many should not be empty")),
            InstructorResponse::Stream(iter) => Err(Error::Generic("Cannot unwrap a stream".to_string())),
        }
    }
}


impl<T> Debug for InstructorResponse<T> 
where T: ValidateArgs<'static> + BaseSchema
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InstructorResponse::One(item) => write!(f, "One({:?})", item),
            InstructorResponse::Many(items) => write!(f, "Many({:?})", items),
            InstructorResponse::Stream(iter) => write!(f, "Stream({:?})", iter.size_hint()),
        }
    }
}

#[derive(Debug, Serialize, Copy, Clone, JsonSchema)]
pub enum IterableOrSingle<T>
where T: ValidateArgs<'static> + BaseSchema
{
    Iterable(T), 
    Single(T),
}

impl<T> IterableOrSingle<T>
where 
    T: ValidateArgs<'static> + BaseSchema
{
    // This method is now correctly placed outside the ValidateArgs trait impl block
    pub fn unwrap(self) -> Result<T, ()> {
        match self {
            IterableOrSingle::Iterable(item) | IterableOrSingle::Single(item) => Ok(item),
        }
    }
}


impl<T> ValidateArgs<'static> for IterableOrSingle<T>
where
    T: ValidateArgs<'static> + BaseSchema,
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

