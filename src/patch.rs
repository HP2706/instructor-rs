
use crate::process_response::handle_response_model;
use crate::iterable::IterableOrSingle;
use crate::retry::retry_sync;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::ChatCompletionRequest;
use crate::traits::BaseSchema;
use validator::ValidateArgs;
use crate::mode::Mode;
use crate::enums::Error;
use crate::enums::InstructorResponse;
use crate::streaming::ChatCompletionResponseWrapper;
use openai_api_rs::v1::error::APIError;
use crate::streaming::{convert_lazy_response, process_streaming_response, collect_stream};

pub trait ChatCompletionStream {
    fn post_stream<T: serde::ser::Serialize>(
        &self,
        path: &str,
        params: &T,
    ) -> Result<ChatCompletionResponseWrapper, APIError>;
}

impl ChatCompletionStream for Client {
    fn post_stream<T: serde::ser::Serialize>(
        &self,
        path: &str,
        params: &T,
    ) -> Result<ChatCompletionResponseWrapper, APIError> {
        let url = format!(
            "{api_endpoint}{path}",
            api_endpoint = self.api_endpoint,
            path = path
        );
        let request = self.build_request(minreq::post(url), false); //is_beta = false
        let res = request.with_json(params).unwrap().send_lazy();
        match res {
            Ok(res) => {
                let status_code = res.status_code.clone();
                if (200..=299).contains(&status_code) {
                    let iterator = convert_lazy_response(res);
                    let stream = process_streaming_response(iterator);
                    Ok(stream)
                } else {
                    let iterator = convert_lazy_response(res);
                    let string_response = collect_stream(iterator);
                    Err(APIError {
                        message: format!("{}: {}", status_code, string_response),
                    })
                }
            }
            Err(e) => Err(APIError {
                message: e.to_string(),
            }),
        }
    }
}

// Define a wrapper type for the Client.
pub struct Patch {
    pub client: Client,
    pub mode: Option<Mode>,
}

impl Patch {
    pub fn chat_completion<T, A>(
        &self, 
        response_model:IterableOrSingle<T>,
        validation_context: A,
        max_retries: usize,
        stream: bool,
        kwargs: ChatCompletionRequest
    ) -> Result<InstructorResponse<A, T>, Error>

    where
        T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
        A: 'static + Copy,
    {
        // if no mode is provided, default to Mode::JSON
        let mode = match self.mode {
            Some(mode) => mode,
            None => Mode::JSON,
        };

        let (response_model, mut kwargs) = handle_response_model(
            response_model, 
            mode, 
            kwargs
        ).map_err(|e| e)?;
        

        let func : Box<dyn Fn(ChatCompletionRequest) -> Result<ChatCompletionResponseWrapper, APIError>> = Box::new(|kwargs| {
            match kwargs.stream {
                Some(false) | None => {
                    match self.client.chat_completion(kwargs) {
                        Ok(res) => Ok(ChatCompletionResponseWrapper::Single(res)),
                        Err(e) => Err(e),
                    }
                },
                Some(true) => {
                    self.client.post_stream("/chat/completions", &kwargs)
                }
            }
            
        });

        return retry_sync(
            func,
            response_model,
            validation_context,
            &mut kwargs,
            max_retries,
            stream,
            self.mode.unwrap(),
        );
    }
    
}








