use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::{GPT4_TURBO_PREVIEW, GPT3_5_TURBO};






fn main() -> Result<(), Box<dyn std::error::Error>> {
    //func_call();

    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            return an essays of 200 words
            ")),
            name: None,
        }],
    ).stream(true);


    

    Ok(())
}


