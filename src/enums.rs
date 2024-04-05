use crate::traits::BaseSchema;
use validator::ValidateArgs;
use crate::error::Error;
use async_openai::types::{CreateChatCompletionResponse, ChatCompletionResponseStream};

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
#[derive(Debug)]
pub enum InstructorResponse<A, T>
    where T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    One(T),
    Many(Vec<T>),
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



