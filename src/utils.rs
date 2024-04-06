use crate::error::Error;
use std::pin::Pin;
use futures::stream::{Stream, StreamExt, iter};
use futures::Future;

use async_openai::types::{
    ChatCompletionResponseMessage, ChatChoice, Role, ChatCompletionMessageToolCall,ChatCompletionToolType, 
    ChatCompletionTool, CreateChatCompletionResponse, FunctionCall
};
pub fn to_sync<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Runtime::new().unwrap().block_on(future)
}

pub fn extract_json_from_codeblock(content: &str) -> Result<String, Error> {
    let first_paren = content.find('{');
    let last_paren = content.rfind('}');

    match (first_paren, last_paren) {
        (Some(start), Some(end)) => Ok(content[start..=end].to_string()),
        _ => Err(Error::JsonExtractionError("No JSON found".to_string())),
    }
}


use async_stream::stream;
use crate::types::JsonStream;

pub async fn extract_json_from_stream_async(
    mut chunks: JsonStream,
) -> JsonStream {
    stream! {
        let mut capturing = false;
        let mut brace_count = 0;
        let mut current_json = String::new();

        while let Some(chunk) = chunks.next().await {
            //TODO could this error be handled better
            for char in chunk.expect("Error extracting json").chars() {
                if char == '{' {
                    capturing = true;
                    brace_count += 1;
                    current_json.push(char);
                } else if char == '}' && capturing {
                    brace_count -= 1;
                    current_json.push(char);
                    if brace_count == 0 {
                        capturing = false;
                        yield Ok(current_json.clone());
                        current_json.clear();
                    }
                } else if capturing {
                    current_json.push(char);
                }
            }
        }
    }.boxed()
}

pub fn create_tool_call(name: String, arguments: String) -> ChatCompletionMessageToolCall {
    ChatCompletionMessageToolCall {
        r#type: ChatCompletionToolType::Function,
        function: FunctionCall {
            name: name,
            arguments : arguments
        },
        id: "id".to_string()
    }
}

pub fn create_chat_completion_choice(
    tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
    content: Option<String>,
) -> ChatChoice {
    ChatChoice {
        index: 0,
        message: ChatCompletionResponseMessage {
            role: Role::Assistant,
            content: content,
            tool_calls: tool_calls,
            function_call: None
        },
        finish_reason: None,
        logprobs : None
    }
}

pub fn create_chat_completion_response(
    tool_calls: Option<Vec<ChatCompletionMessageToolCall>>,
    content: Option<String>,
) -> CreateChatCompletionResponse {
    let choices = vec![create_chat_completion_choice(tool_calls, content)];
    CreateChatCompletionResponse {
        id: "chat.completion.chunk".to_string(),
        object: "chat.completion".to_string(),
        created: 0 as u32,
        model: "gpt-4-turbo-preview".to_string(),
        choices: choices,
        usage: None,
        system_fingerprint: None
    }
}
