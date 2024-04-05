/* use validator::ValidateArgs;
use crate::error::Error;
use crate::iterable::IterableOrSingle;
use crate::mode::Mode;
use crate::traits::BaseSchema;
use crate::traits::OpenAISchema;
use crate::utils::extract_json_from_stream;
use crate::enums::{InstructorResponse, ChatCompletionResponseWrapper};
use async_openai::types::{ChatCompletionResponseStream, ChatCompletionStreamResponseDelta};
use async_openai::error::OpenAIError;

pub trait IterableBase<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + BaseSchema<T>,
    Args: 'static + Copy,
{
    type Args : 'static + Copy;

    fn extract_json(
        completion : ChatCompletionResponseStream,
        mode : Mode
    ) -> Box< dyn Iterator<Item = Result<String, Error>>>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;

    fn from_streaming_response(
        model: &IterableOrSingle<Self>,
        response: ChatCompletionResponseStream,
        validation_context: &Args,
        mode: Mode,
    ) -> InstructorResponse<Args, T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;
    
    fn tasks_from_chunks(
        model: &IterableOrSingle<Self>,
        json_chunks: Box<dyn Iterator<Item = Result<String, StreamingError>>>,
        validation_context: &Args
    ) -> InstructorResponse<Args, T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;
    
    fn tasks_from_chunks(
        model: &IterableOrSingle<Self>,
        json_chunks: Box<dyn Iterator<Item = Result<String, Error>>>,
        validation_context: &Args
    ) -> InstructorResponse<Args, T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>;

    fn get_object(s: &str, index: usize) -> (Option<String>, String);
}


impl<A, T> IterableBase<A, T> for T
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    type Args = A;

    fn extract_json(
        completion : ChatCompletionResponseStream,
        mode : Mode
    ) -> Box< dyn Iterator<Item = Result<String, Error>>>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>
    {
        Box::new(completion.filter_map(move |chunk_result| {
            match chunk_result {
                Ok(chunk) => match chunk {
                    ChatCompletionResponseStream::CreateChatCompletionStreamResponse(chunk) => {
                        match mode {
                            Mode::JSON | Mode::MD_JSON | Mode::JSON_SCHEMA => {
                                chunk.choices.get(0).and_then(|choice| {
                                    match &choice.delta.content {
                                        Some(content) => {
                                            Some(Ok(content.clone()))
                                        },
                                        None => None,
                                    }
                                })
                            },
                            Mode::TOOLS => {
                                //TODO: Implement this (check openai api)
                                Some(Err(Error::Generic(
                                    format!("Mode {:?} is not supported for MultiTask streaming", mode)
                                )))
                            },
                            _ => Some(Err(Error::Generic(
                                format!("Mode {:?} is not supported for MultiTask streaming", mode)
                            ))),
                        }
                    },
                    ChatCompletionResponseStream::OpenAIError(e) => None, //todo think about this
                },
                Err(e) => Some(Err(e)),
            }
        }))
    }

    fn from_streaming_response(
        model: &IterableOrSingle<Self>,
        response: ChatCompletionResponseStream,
        validation_context: &Self::Args,
        mode: Mode,
    ) -> InstructorResponse<Self::Args, T>
    { 

        let mut json_chunks  = Self::extract_json(response, mode);

        if mode == Mode::MD_JSON {
            json_chunks = extract_json_from_stream(json_chunks);
        }

        Self::tasks_from_chunks(model, json_chunks, validation_context)
    }

    fn tasks_from_chunks(
        model: &IterableOrSingle<Self>,
        json_chunks: Box<dyn Iterator<Item = Result<String, Error>>>,
        validation_context: &Self::Args
    ) -> InstructorResponse<Self::Args, T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema<T>
    {
        let mut started = false;
        let mut potential_object = String::new();
        let stream = json_chunks.flat_map(move |chunk_result| {
            match chunk_result {
                Ok(chunk) => {
                    potential_object.push_str(&chunk);
                    if !started {
                        if let Some(index) = chunk.find('[') {
                            started = true;
                            potential_object = chunk[index + 1..].to_string();
                        }
                        ChatCompletionStreamingResponse::Done => {
                            println!("{:?}", "done");
                        }
                        return None;
                    }
                },
                Err(e) => return Some(Err(e)),
            }

            let (task_json, new_potential_object) = Self::get_object(&potential_object, 0);
            potential_object = new_potential_object;

            if let Some(task_json) = task_json {
                match Self::model_validate_json(model, &task_json, validation_context) {
                    Ok(single) => {
                        let model = single.unwrap().unwrap().unwrap();
                        Some(Ok(model))
                    },
                    Err(e) => Some(Err(Error::ModelValidationError(e.to_string()))),
                }
            } else {
                None
            }
        });
    
        InstructorResponse::Stream(Box::new(stream))
    }

    fn get_object(s: &str, mut stack: usize) -> (Option<String>, String) {
        let start_index = s.find('{');
        if let Some(start) = start_index {
            for (i, c) in s.char_indices() {
                if c == '{' {
                    stack += 1;
                } else if c == '}' {
                    stack -= 1;
                    if stack == 0 {
                        // Adjusted slicing to handle Rust's string slice indexing
                        return (Some(s[start..=i].to_string()), s[i+1..].to_string());
                    }
                }
                Err(e) => eprintln!("Error: {:?}", e),
            }
        });
    
        InstructorResponse::Stream(Box::new(stream))
    }

    fn get_object(s: &str, mut stack: usize) -> (Option<String>, String) {
        let start_index = s.find('{');
        if let Some(start) = start_index {
            for (i, c) in s.char_indices() {
                if c == '{' {
                    stack += 1;
                } else if c == '}' {
                    stack -= 1;
                    if stack == 0 {
                        // Adjusted slicing to handle Rust's string slice indexing
                        return (Some(s[start..=i].to_string()), s[i+1..].to_string());
                    }
                }
            }
        }
        (None, s.to_string())
    }
}
 */