use crate::error::Error;

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

pub fn extract_json_from_stream(
    chunks: Box<dyn Iterator<Item = Result<String, Error>>>,
) -> Box<dyn Iterator<Item = Result<String, Error>>> {
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


