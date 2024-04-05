use validator::ValidationErrors;
use std::fmt;
use std::error::Error;
use derive_builder::Builder;
use async_openai::types::{CreateChatCompletionRequestArgs, CreateChatCompletionRequest};


pub trait ToBuilder<T>
where
    T: Default,
{
    fn to_builder(&self) -> T;
}

impl ToBuilder<CreateChatCompletionRequestArgs> for CreateChatCompletionRequest {
    fn to_builder(&self) -> CreateChatCompletionRequestArgs {
        CreateChatCompletionRequest::default()
            .messages(self.messages.clone())
            .model(self.model.clone())
            .temperature(self.temperature)
            .max_tokens(self.max_tokens)
            .top_p(self.top_p)
            .frequency_penalty(self.frequency_penalty)
            .presence_penalty(self.presence_penalty)
            .stop(self.stop.clone())
            .stream(self.stream)
            .n(self.n)
            .logit_bias(self.logit_bias.clone())
            .logprobs(self.logprobs)
            .top_logprobs(self.top_logprobs)
            .response_format(self.response_format.clone())
            .seed(self.seed)
            .tools(self.tools.clone())
            .tool_choice(self.tool_choice.clone())
            .user(self.user.clone())
            .function_call(self.function_call.clone())
            .functions(self.functions.clone())
    }
}

#[derive(Debug)]
pub enum JsonError {
    Validation(ValidationErrors),
    Generic(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JsonError::Validation(ref err) => write!(f, "Validation error: {}", err),
            JsonError::Generic(ref err) => write!(f, "Error: {}", err),
        }
    }
}

impl From<ValidationErrors> for JsonError {
    fn from(err: ValidationErrors) -> JsonError {
        JsonError::Validation(err)
    }
}


#[derive(Debug)]
pub struct RetryError {
    pub last_attempt: Box<dyn Error>, // Simplified for example purposes
}

