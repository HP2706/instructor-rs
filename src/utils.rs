use syn::ItemFn;
use crate::enums::Error;
use openai_api_rs::v1::chat_completion::{
    ToolCall, ToolCallFunction, 
    MessageRole, ChatCompletionChoice, ChatCompletionResponse, ChatCompletionMessageForResponse
};
use openai_api_rs::v1::common::Usage;
use crate::streaming::StreamingError;


pub fn is_async(func: &ItemFn) -> bool {
    func.sig.asyncness.is_some()
}

pub fn extract_json_from_codeblock(content: &str) -> Result<String, Error> {
    let first_paren = content.find('{');
    let last_paren = content.rfind('}');

    match (first_paren, last_paren) {
        (Some(start), Some(end)) => Ok(content[start..=end].to_string()),
        _ => Err(Error::JsonExtractionError("No JSON found".to_string())),
    }
}

pub fn extract_json_from_stream(
    chunks: Box<dyn Iterator<Item = Result<String, StreamingError>>>,
) -> Box<dyn Iterator<Item = Result<String, StreamingError>>> {
    let mut capturing = false;
    let mut brace_count = 0;
    let mut json_accumulator = String::new();

    Box::new(chunks.flat_map(move |chunk_result| {
        match chunk_result {
            Ok(chunk) => chunk.chars().map(Ok).collect::<Vec<_>>(),
            Err(e) => vec![Err(e)],
        }
    }).filter_map(move |result| {
        match result {
            Ok(char) => {
                if char == '{' {
                    if !capturing {
                        json_accumulator.clear(); // Start a new capture
                    }
                    capturing = true;
                    brace_count += 1;
                } else if char == '}' && capturing {
                    brace_count -= 1;
                }

                if capturing {
                    json_accumulator.push(char);
                    if brace_count == 0 {
                        capturing = false;
                        return Some(Ok(json_accumulator.clone())); // Return the captured JSON string
                    }
                }
                None
            },
            Err(_) => Some(result.map(|_| json_accumulator.clone())), // Pass through errors
        }
    }))
}

///these are for better testing

pub fn create_tool_call(name : String, arguments : String) -> ToolCall {
    ToolCall { 
        r#type : "function".to_string(),
        id: "call_UhIRWDIKUO3kARySeidFH7lb".to_string(),
        function: ToolCallFunction { 
            name: Some(name), 
            arguments: Some(arguments)
        } 
    }
}

pub fn create_chat_completion_choice(tool_calls: Option<Vec<ToolCall>>, content : Option<String>) -> ChatCompletionChoice {
    ChatCompletionChoice { 
        index: 0, 
        message: ChatCompletionMessageForResponse { 
            role: MessageRole::assistant, 
            content: content, 
            name: None, 
            tool_calls: tool_calls
        },
        finish_details : None,
        finish_reason : None
    }
}



pub fn create_chat_completion_response(tool_calls: Option<Vec<ToolCall>>, content : Option<String>) -> ChatCompletionResponse {
    let choices = vec![create_chat_completion_choice(tool_calls, content)];
    return ChatCompletionResponse {
        id: "123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "gpt-4".to_string(),
        choices: choices,
        usage: Usage {
            prompt_tokens: 100,
            completion_tokens: 100,
            total_tokens: 200,
        },
        system_fingerprint: None,
        headers: None,
    };
}


