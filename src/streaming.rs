///this is intermediate work for building streaming support 
use std::fmt::{Display, Debug};
use std::collections::HashMap;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::de::{self, Visitor, MapAccess};
use std::fmt;
use serde::ser::SerializeMap;
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Delta {
    Content { content: String },
    Empty {},
}

impl Delta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Delta::Content { ref content } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("content", content)?;
                map.end()
            }
            Delta::Empty {} => {
                let map = serializer.serialize_map(Some(0))?;
                map.end()
            }
        }
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DeltaVisitor;

        impl<'de> Visitor<'de> for DeltaVisitor {
            type Value = Delta;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Delta")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Delta, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut content = None;
                while let Some(key) = map.next_key::<String>()? {
                    if key == "content" {
                        if content.is_some() {
                            return Err(de::Error::duplicate_field("content"));
                        }
                        content = Some(map.next_value()?);
                    } else {
                        return Err(de::Error::unknown_field(&key, &["content"]));
                    }
                }
                Ok(match content {
                    None => Delta::Empty {},
                    Some(content) => Delta::Content { content },
                })
            }
        }

        deserializer.deserialize_any(DeltaVisitor)
    }
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ChatCompletionChoiceStreaming {
    pub index: i64,
    pub delta: Delta,
    pub logprobs: Option<i32>,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ChatCompletionStreamingResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoiceStreaming>,
    pub system_fingerprint: Option<String>,
    pub headers: Option<HashMap<String, String>>,
}




#[derive()]
pub enum ChatCompletionResponseWrapper {
    Single(ChatCompletionResponse),
    Stream(Box<dyn Iterator<Item = Result<ChatCompletionStreamingChunk, StreamingError>>>),
}

#[derive(Debug)]
pub enum StreamingError {
    IoError(std::io::Error),
    MinreqError(minreq::Error),
    JsonError(serde_json::Error),
    Generic(String),
}

impl Display for StreamingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamingError::IoError(e) => write!(f, "IoError({})", e),
            StreamingError::MinreqError(e) => write!(f, "MinreqError({})", e),
            StreamingError::JsonError(e) => write!(f, "JsonError({})", e),
            StreamingError::Generic(e) => write!(f, "Own({})", e),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChatCompletionStreamingChunk {
    Chunk(ChatCompletionStreamingResponse),
    Done,
}




fn convert(response: minreq::ResponseLazy) -> Box<dyn Iterator<Item = Result<(u8, usize), StreamingError>>> {
    let iterator = response.into_iter().enumerate().map(|(len, result)| {
        result.map(|byte| byte)
        .map_err(|e| StreamingError::MinreqError(e))
    });
    Box::new(iterator)
}

fn process_streaming_response(
    stream: Box<dyn Iterator<Item = Result<(u8, usize), StreamingError>>>
) -> ChatCompletionResponseWrapper {
    let mut buffer = String::new();
    let stream = stream.filter_map(move |result| {
        match result {
            Ok((byte, _)) => {
                buffer.push(byte as char);
                if buffer.contains("data: {") && buffer.contains("}\n") {
                    let start = buffer.find("data: {").unwrap() + 6;
                    let end = buffer.find("}\n").unwrap() + 1;
                    let json = buffer[start..end].to_string();
                    buffer.clear();
                    Some(extract_json(&json))
                } else if buffer.contains("data: [DONE]") {
                    buffer.clear();
                    Some(Ok(ChatCompletionStreamingChunk::Done))
                } else {
                    None // Do not yield a value, effectively skipping this iteration
                }
            },
            Err(e) => Some(Err(StreamingError::Generic(e.to_string()))),
        }
    });
    ChatCompletionResponseWrapper::Stream(Box::new(stream))
}


fn extract_json(json_data: &str) -> Result<ChatCompletionStreamingChunk, StreamingError> {
    if json_data.trim() == "[DONE]" {
        return Ok(ChatCompletionStreamingChunk::Done);
    }
    let json_value = serde_json::from_str::<ChatCompletionStreamingResponse>(json_data);
    match json_value {
        Ok(value) => {
            // Process the JSON data as needed
            Ok(ChatCompletionStreamingChunk::Chunk(value))
        }
        Err(e) => {
            return Err(StreamingError::Generic(format!("json error on:\n{}\nerror message\n{}", json_data, e.to_string())))
        },
    }
}


fn cached_streamer() -> impl Iterator<Item = Result<(u8, usize), StreamingError>> {
    let text = r#"
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"role":"assistant","content":""},"logprobs":null,"finish_reason":null}]}

    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"content":"In"},"logprobs":null,"finish_reason":null}]}
    
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"content":"novation"},"logprobs":null,"finish_reason":null}]}
    
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"content":","},"logprobs":null,"finish_reason":null}]}
    
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"content":" Collaboration"},"logprobs":null,"finish_reason":null}]}
    
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"content":","},"logprobs":null,"finish_reason":null}]}
    
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{"content":" Sustainability"},"logprobs":null,"finish_reason":null}]}
    
    data: {"id":"chatcmpl-9A0hcaguAav3PrvNA54IFivY2API7","object":"chat.completion.chunk","created":1712173008,"model":"gpt-4-0125-preview","system_fingerprint":"fp_b77cb481ed","choices":[{"index":0,"delta":{},"logprobs":null,"finish_reason":"stop"}]}
    
    data: [DONE]"#;
    text.bytes().enumerate().map(|(index, byte)| Ok((byte, index))).collect::<Vec<Result<(u8, usize), StreamingError>>>().into_iter()
}