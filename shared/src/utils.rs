
pub fn extract_json_from_codeblock(content: &str) -> String {
    let first_paren = content.find('{');
    let last_paren = content.rfind('}');

    match (first_paren, last_paren) {
        (Some(start), Some(end)) => content[start..=end].to_string(),
        _ => String::new(),
    }
}