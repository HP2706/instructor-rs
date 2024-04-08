
use crate::process_response::handle_response_model;
use crate::enums::IterableOrSingle;
use crate::retry::retry_async;
use async_openai::types::CreateChatCompletionRequest;
use std::marker::{Send, Sync};
use async_openai::Client;
use async_openai::error::OpenAIError;
use async_openai::config::Config;
use crate::openai_schema::{BaseSchema, BaseArg};
use validator::ValidateArgs;
use crate::mode::Mode;
use crate::error::Error;
use crate::enums::{InstructorResponse, ChatCompletionResponseWrapper};
use std::pin::Pin;
use std::future::Future;
// Define a wrapper type for the Client.

#[derive(Debug, Clone)]
pub struct Patch<C: Config> {
    pub client: Client<C>,
    pub mode: Option<Mode>,
}


impl<C> Patch<C> 
where
    C: Config + Clone + Send + Sync + 'static,
{
    /// Initiates a chat completion request with the OpenAI API.
    /// 
    /// # Arguments
    /// 
    /// * `response_model`: `IterableOrSingle<T>` - an enum wrapper that is very similar to Iterable[model] in instructor
    ///     either use IterableOrSingle::Iterable<T> or IterableOrSingle::Single<T>.
    ///     T must implements the traits necessary for using OpenAISchema and IterableBase
    /// * `validation_context`: `A` - The context or data used for validating the request. 
    ///   The type `A` must implement the `BaseArg` trait.
    /// * `max_retries`: `usize` - The maximum number of retries for the request in case of failures.
    /// * `kwargs`: `CreateChatCompletionRequest` - Additional keyword arguments to customize the chat completion request, 
    ///     This is the object you would have sent to openai. via client.chat().create(request)
    /// 
    /// # Returns
    /// 
    /// A `Result` type that, on success, contains an `InstructorResponse<T>`, 
    /// which wraps the response model(s) in the specified format (either single or iterable). 
    /// On failure, it returns an `Error`.
    /// 
    /// # Examples
    /// 
    /// ```
    /// let client = Client::new(); //defaults to env variable OPENAI_API_KEY
    /// let patch = Patch::new(client);
    /// let response = patch.chat_completion(IterableOrSingle::Iterable(MyModel::default()), MyValidationContext, 3, CreateChatCompletionRequest::default()).await?;
    /// ```
    pub async fn chat_completion<T, A>(
        &self, 
        response_model:IterableOrSingle<T>,
        validation_context: A,
        max_retries: usize,
        kwargs: CreateChatCompletionRequest
    ) -> Result<InstructorResponse<T>, Error>
    where
        T: ValidateArgs<'static, Args=A> + BaseSchema + 'static,
        A: BaseArg,
    {

        let mut kwargs = kwargs.clone();
        // if no mode is provided, default to Mode::JSON
        let mode = match self.mode {
            Some(mode) => mode,
            None => Mode::JSON,
        };

        handle_response_model(
            &response_model, 
            mode, 
            &mut kwargs
        ).map_err(|e| e)?;
        
        let client = self.client.clone();
        let func: Box<
            dyn Fn(CreateChatCompletionRequest) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponseWrapper, OpenAIError>> + Send>>
                + Send
                + Sync,
        > = Box::new(move |kwargs| {
            let client = client.clone();
            Box::pin(async move {
                match kwargs.stream {
                    Some(false) | None => {
                        let res = client.chat()
                        .create(kwargs).await?;
                        Ok(ChatCompletionResponseWrapper::AtOnce(res))
                    }
                    Some(true) => {
                        let res = client.chat()
                        .create_stream(kwargs).await?;
                        Ok(ChatCompletionResponseWrapper::Stream(res))
                    }
                }
            })
        });

        retry_async(
            func,
            response_model,
            validation_context,
            &mut kwargs,
            max_retries,
            self.mode.unwrap(),
        ).await
    }
    
}








