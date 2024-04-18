use validator::ValidateArgs;
use crate::error::Error;
use crate::enums::IterableOrSingle;
use crate::mode::Mode;
use crate::openai_schema::BaseArg;
use crate::openai_schema::BaseSchema;
use crate::openai_schema::OpenAISchema;
use crate::enums::InstructorResponse;
use async_openai::types::{ChatCompletionResponseStream};
use std::pin::Pin;
use crate::types::JsonStream;
use futures::stream::{Stream, StreamExt};
use async_stream::stream;
use pin_utils::pin_mut;


///This is the base trait for parsing streaming responses
/// in order to use with your struct, your strutc must implement the following traits:
/// JsonSchema, Serialize, Debug, Default, Validate, Deserialize, Clone
/// this can be done either via #[derive(JsonSchema, Serialize, Debug, Default, Validate, Deserialize, Clone)] or by importing
/// model_traits_macro::derive_all and calling [derive_all] on your struct
/// 
/// Example
/// 
/// #[derive_all]
/// struct MyStruct {
///     a: i32,
///     b: i32,
/// }
/// 
/// now you can access the following methods:
/// 
/// Mystruct::from_streaming_response_async(...)

pub trait IterableBase<Args, T> 
where
    T: ValidateArgs<'static, Args=Args> + BaseSchema + 'static ,
    Args: BaseArg,
{
    type Args : BaseArg;

    ///recieves a stream of CreateChatCompletionStreamResponse and returns a stream of strings
    /// 
    /// # Arguments
    /// 
    /// * `completion` - A stream of CreateChatCompletionStreamResponse
    /// * `mode` - The mode to extract the json in
    /// 
    /// # Returns
    /// 
    /// A stream of strings and or Errors
    async fn extract_json_async(
        completion : ChatCompletionResponseStream,
        mode : Mode
    ) ->  Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;

    ///recieves a stream of CreateChatCompletionStreamResponse and returns a stream of parsed objects 
    /// for each object T_i in the stream T_i::model_validate_json() is called on it 
    /// 
    /// # Arguments
    /// 
    /// * `model` - The model to use
    /// * `response` - The stream of CreateChatCompletionStreamResponse
    /// * `validation_context` - The validation context to use 
    /// (this is if you have validator functions that require custom context exactly like validation_context in pydantic)
    /// * `mode` - The mode to extract the json in 
    /// 
    /// # Returns
    /// * `InstructorResponse::Stream(stream)` - A stream of Result<T, Error> where T is the parsed struct
    async fn from_streaming_response_async(
        model: IterableOrSingle<Self>,
        response: ChatCompletionResponseStream,
        validation_context: &Args,
        mode: Mode,
    ) -> InstructorResponse<T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;
    
    ///recieves a stream of strings(JsonStream) and returns a stream of Result<T, Error> where T is the parsed struct 
    /// for each object T_i in the stream T_i::model_validate_json() is called on it.
    /// self.get_object() is used to collect the tokens into a string that can get parsed as json
    /// 
    /// # Arguments
    /// 
    /// * `model` - The model to use
    /// * `response` - The stream of CreateChatCompletionStreamResponse
    /// * `validation_context` - The validation context to use 
    /// (this is if you have validator functions that require custom context exactly like validation_context in pydantic)
    /// * `mode` - The mode to extract the json in 
    /// 
    /// # Returns
    /// * `InstructorResponse::Stream(stream)` - A stream of Result<T, Error> where T is the parsed struct    
    async fn tasks_from_chunks_async(
        model: IterableOrSingle<Self>,
        json_chunks: JsonStream,
        validation_context: Args
    ) -> InstructorResponse<T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema;

    fn get_object(s: &str, index: usize) -> (Option<String>, String);
}

impl<A, T> IterableBase<A, T> for T
where
    T: ValidateArgs<'static, Args=A> + BaseSchema + 'static ,
    A: BaseArg + 'static,
{
    type Args = A;

    async fn extract_json_async(
        completion: ChatCompletionResponseStream,
        mode: Mode
    ) -> Pin<Box<dyn Stream<Item = Result<String, Error>> + Send>>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema
    {
        let t0 = std::time::Instant::now();
        let stream = completion.filter_map(move |chunk_result| {
            async move {
                match chunk_result {
                    Ok(chunk) => {
                        // Assuming each chunk or its relevant parts can be cloned as needed
                        match mode {
                            Mode::JSON | Mode::MD_JSON | Mode::JSON_SCHEMA => {
                                let chunk = chunk.choices.get(0).and_then(|choice| {
                                    // Here, we clone the content of the choice, if necessary
                                    choice.delta.content.clone().map(
                                        |content| Ok(Ok(content))
                                    )
                                });
                                println!("json: chunk: {:?} at time: {:?}", chunk, t0.elapsed());
                                chunk
                            },
                            Mode::TOOLS => {
                                if let Some(choice) = chunk.choices.get(0) {
                                    if let Some(arguments) = choice.delta.tool_calls.as_ref()
                                        .and_then(|tc| tc.get(0))
                                        .and_then(|call| call.function.as_ref())
                                        .and_then(|f| f.arguments.clone()) {
                                        return Some(Ok(Ok(arguments)));
                                    }
                                }
                                None // else we return None
                            },
                            _ => Some(Err(Error::Generic(
                                format!("Mode {:?} is not supported for MultiTask streaming", mode)
                            ))),
                        }
                    },
                    Err(_) => None, // Consider how to handle stream errors appropriately
                }
            }
        }).flat_map(|option| futures::stream::iter(option.into_iter()));
        Box::pin(stream)
    }

    async fn from_streaming_response_async(
        model: IterableOrSingle<Self>,
        response: ChatCompletionResponseStream,
        validation_context: &Self::Args,
        mode: Mode,
    ) -> InstructorResponse<T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema
    { 

        let json_chunks  = Self::extract_json_async(response, mode).await;
        //if mode == Mode::MD_JSON {
        //    let mut json_chunks: JsonStream = extract_json_from_stream_async(response).await;
        //} 
        Self::tasks_from_chunks_async(model, json_chunks, validation_context.clone()).await
    }

    async fn tasks_from_chunks_async(
        model: IterableOrSingle<Self>,
        json_chunks: JsonStream,
        validation_context: Self::Args,
    ) -> InstructorResponse<T>
    where
        Self: Sized + ValidateArgs<'static> + BaseSchema,
    {
        let mut started = false;
        let mut potential_object = String::new();
        let stream = stream! {
            pin_mut!(json_chunks); // Ensure json_chunks is pinned for .next() in async context
            while let Some(chunk_result) = json_chunks.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        potential_object.push_str(&chunk);
                        if !started {
                            if let Some(index) = chunk.find('[') {
                                started = true;
                                potential_object = chunk[index + 1..].to_string();
                                continue; // Continue to the next iteration of the loop
                            }
                        }
                        let (task_json, new_potential_object) = Self::get_object(&potential_object, 0);
                        potential_object = new_potential_object;
                        if let Some(task_json) = task_json {
                            // Ensure model_validate_json and its entire call chain are `Send`
                            match Self::model_validate_json(&model, &task_json, &validation_context) {
                                Ok(single) => {
                                    yield Ok(single.unwrap().unwrap());
                                },
                                Err(e) => {
                                    yield Err(e);
                                },
                            }
                        }
                    },
                    Err(e) => {
                        yield Err(e);
                    },
                }
            }
        }.boxed(); // If you're using tokio, you might need to use .boxed().send() here
        InstructorResponse::Stream(stream)
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
