use instructor_rs::utils::{extract_json_from_codeblock, string_to_stream, extract_json_from_stream_async };
use instructor_rs::error::Error;
use futures::stream::{self, StreamExt}; // Ensure StreamExt is imported for collect()
use instructor_rs::types::JsonStream;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy_fail() {
        assert_eq!(1, 2); // this is stupid but cargo wont show the test as failed if all pass
    }

    #[test]
    fn test_extract_json_from_codeblock_success() {
        let content = "Some text {\"key\": \"value\"} more text";
        let result = extract_json_from_codeblock(content);
        assert_eq!(result.unwrap(), "{\"key\": \"value\"}");
    }

    #[test]
    fn test_extract_json_from_codeblock_no_json() {
        let content_no_json = "No JSON here!";
        let result = extract_json_from_codeblock(content_no_json);
        assert!(result.is_err());
        if let Err(Error::JsonExtractionError(msg)) = result {
            assert_eq!(msg, "No JSON found");
        } else {
            panic!("Expected JsonExtractionError");
        }
    }

    #[test]
    fn test_extract_json_from_codeblock_multiple_json_objects() {
        let content = "JSON {\"key1\": \"value1\"},{\"key2\": \"value2\"}";
        let result = extract_json_from_codeblock(content);
        println!("result: {:?}", result);
        let correct = "{\"key1\": \"value1\"},{\"key2\": \"value2\"}".to_string();
        assert_eq!(result.unwrap(), correct);
    }

    #[test]
    fn test_extract_json_from_codeblock_nested_json() {
        let content = "Nested JSON {\"key\": {\"nestedKey\": \"nestedValue\"}}";
        let result = extract_json_from_codeblock(content);
        assert_eq!(result.unwrap(), "{\"key\": {\"nestedKey\": \"nestedValue\"}}");
    }

    #[tokio::test]
    
    async fn test_extract_json_from_stream_async() {
        let text = r#"here is the json for you! 
    
        ```json
        , here
        {
            "key": "value",
            "another_key": [{"key": {"key": "value"}}]
        }
        ```
        What do you think?
        "#;

        #[derive(serde::Deserialize)]
        struct InnerKey {
            key: String,
        }

        #[derive(serde::Deserialize)]
        struct AnotherKey {
            key: InnerKey, // Adjusted to match the nested structure
        }

        #[derive(serde::Deserialize)]
        struct Json {
            key: String,
            another_key: Vec<AnotherKey>,
        }


        let json_stream = string_to_stream(text.to_string()).await;
        let stream = extract_json_from_stream_async(json_stream).await;
        let results: Vec<Result<String, _>> = stream.collect().await; // Collect into Vec<Result<String, _>>
        let collected: String = results.into_iter().collect::<Result<Vec<_>, _>>().unwrap().join(" "); // Handle errors and concatenate
        let json: Json = serde_json::from_str(&collected).unwrap();
        assert_eq!(json.key, "value");
    }
    
    #[tokio::test]
    async fn test_multiple_extract_json_from_stream_async() {
        let text = r#"{'key1': 'value'}, {'key2': 'value'}"#;
        let json_stream = string_to_stream(text.to_string()).await;
        let stream = extract_json_from_stream_async(json_stream).await;
        let results: Vec<Result<String, _>> = stream.collect().await; // Collect into Vec<Result<String, _>>
        let collected: String = results.into_iter().collect::<Result<Vec<_>, _>>().unwrap().join(" ");// Await the collect() call
        // Check that both JSON objects are present in the output
        let expected_1 = "{'key1': 'value'}";
        let expected_2 = "{'key2': 'value'}";
        assert!(collected.contains(expected_1), "Output does not contain the first expected JSON object.");
        assert!(collected.contains(expected_2), "Output does not contain the second expected JSON object.");
    }
}