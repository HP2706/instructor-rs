use crate::traits::BaseSchema;
use validator::{ValidateArgs, ValidationErrors};
use crate::error::Error;
use async_openai::types::{CreateChatCompletionResponse, ChatCompletionResponseStream};
use std::pin::Pin;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;


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
pub enum InstructorResponse<A, T>
    where T: ValidateArgs<'static, Args=A> + BaseSchema,
    A: 'static + Copy,
{
    One(T),
    Many(Vec<T>),
    Stream(Pin<Box<dyn Stream<Item = Result<T, Error>> + Send>>)
}

pub enum MaybeStream<T> {
    Stream(Pin<Box<dyn Stream<Item = Result<T, Error>> + Send>>),
    One(T),
    Many(Vec<T>),
}

impl<A, T> InstructorResponse<A, T>
where
    T: ValidateArgs<'static, Args = A> + BaseSchema,
    A: 'static + Copy,
{
    pub fn unwrap(self) -> Result<T, Error> {
        match self {
            InstructorResponse::One(item) => Ok(item),
            InstructorResponse::Many(mut items) => Ok(items.pop().expect("InstructorResponse::Many should not be empty")),
            InstructorResponse::Stream(iter) => Err(Error::Generic("Cannot unwrap a stream".to_string())),
        }
    }
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
*/


#[derive(Debug, Serialize, Copy, Clone, JsonSchema)]
pub enum IterableOrSingle<T>
where T: ValidateArgs<'static>
{
    Iterable(T), 
    Single(T),
}

impl<T> IterableOrSingle<T>
where 
    T: ValidateArgs<'static> 
{
    // This method is now correctly placed outside the ValidateArgs trait impl block
    pub fn unwrap(self) -> T {
        match self {
            IterableOrSingle::Iterable(item) | IterableOrSingle::Single(item) => item,
        }
    }
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

