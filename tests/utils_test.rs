use instructor_rs::utils::extract_json_from_codeblock;
use instructor_rs::enums::Error;

#[cfg(test)]
mod tests {
    use super::*;

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
        let content = "JSON {\"key1\": \"value1\"} and another JSON {\"key2\": \"value2\"}";
        let result = extract_json_from_codeblock(content);
        println!("result: {:?}", result);
        assert_eq!(result.unwrap(), "{\"key1\": \"value1\"}");
    }

    #[test]
    fn test_extract_json_from_codeblock_nested_json() {
        let content = "Nested JSON {\"key\": {\"nestedKey\": \"nestedValue\"}}";
        let result = extract_json_from_codeblock(content);
        assert_eq!(result.unwrap(), "{\"key\": {\"nestedKey\": \"nestedValue\"}}");
    }
}