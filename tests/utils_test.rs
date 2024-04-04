use instructor_rs::utils::{extract_json_from_codeblock, extract_json_from_stream};
use instructor_rs::enums::Error;

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

    #[test]
    
    fn test_extract_json_from_stream() {
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


        let chunks = text.split(' ');
        let json_stream = extract_json_from_stream(chunks);
        let collected: String = json_stream.collect();
        println!("collected: {:?}", collected);
        let json: Json = serde_json::from_str(&collected).unwrap();
        assert_eq!(json.key, "value");
    }
    
    #[test]
    fn test_multiple_extract_json_from_stream() {
        let input = r#"{'key1': 'value'}, {'key2': 'value'}"#;
        let chunks = input.split(','); // Directly split without trimming
        let json_stream = extract_json_from_stream(chunks); // Assuming extract_json_from_stream can accept an Iterator<Item=&str>
        let collected: String = json_stream.collect();
        
        // Check that both JSON objects are present in the output
        let expected_1 = "{'key1': 'value'}";
        let expected_2 = "{'key2': 'value'}";
        assert!(collected.contains(expected_1), "Output does not contain the first expected JSON object.");
        assert!(collected.contains(expected_2), "Output does not contain the second expected JSON object.");
    }
}