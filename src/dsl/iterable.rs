use schemars::JsonSchema;
use validator::ValidateArgs;
use serde::{Deserialize, Serialize};
use openai_api_rs::v1::chat_completion::{ChatCompletionResponse, ToolCall};
use crate::enums::{
    Error, InstructorResponse
};
use openai_api_rs::v1::chat_completion::{
    Function, FunctionParameters, JSONSchemaDefine, JSONSchemaType
};
use crate::streaming::StreamingError;
use crate::iterable::IterableOrSingle;
use crate::mode::Mode;
use crate::utils::extract_json_from_stream;
use std::collections::HashMap;
use std::fmt::Debug;
use std::any::type_name;
use crate::traits::BaseSchema;
use crate::streaming::{ChatCompletionResponseWrapper, ChatCompletionStreamingResponse};

pub trait IterableBase<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + BaseSchema<T>,
    Args: 'static + Copy,
{
    type Args : 'static + Copy;
    fn from_streaming_response(
        model: &IterableOrSingle<Self>,
        response: Box<dyn Iterator<Item = Result<ChatCompletionStreamingResponse, StreamingError>>>,
        validation_context: &Args,
        mode: Mode,
    ) -> Result<(), Error>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;
    }


impl<A, T> IterableBase<A, T> for T
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    type Args = A;
    fn from_streaming_response(
        model: &IterableOrSingle<Self>,
        response: Box<dyn Iterator<Item = Result<ChatCompletionStreamingResponse, StreamingError>>>,
        validation_context: &Self::Args,
        mode: Mode,
    ) -> Result<(), Error> {//Result<InstructorResponse<Self::Args, T>, Error>{

        
        for chunk in response {
            match chunk {
                Ok(chunk) => {
                    match chunk {
                        ChatCompletionStreamingResponse::Chunk(choice) => {
                            println!("{:?}", choice);
                        }
                        ChatCompletionStreamingResponse::Done => {
                            println!("{:?}", "done");
                        }
                    }
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }
        }
        Ok(())
        /* let mut chunks = response.into_iter();
        if mode == Mode::MD_JSON {
            return extract_json_from_stream(chunks);
        }
        
        
        Err(Error::NotImplementedError(
            "This feature is not yet implemented.".to_string(),
        )) */
    }
}
