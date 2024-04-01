use syn::ItemFn;
use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json;

pub fn Schema_To_String<'v_a, T>() -> String 
    where T: JsonSchema + Serialize 
{
    let schema = schema_for!(T);
    let schema_string = serde_json::to_string_pretty(&schema).unwrap();
    return schema_string;
}

pub fn is_async(func: &ItemFn) -> bool {
    func.sig.asyncness.is_some()
}

pub fn extract_json_from_codeblock(content: &str) -> String {
    let first_paren = content.find('{');
    let last_paren = content.rfind('}');

    match (first_paren, last_paren) {
        (Some(start), Some(end)) => content[start..=end].to_string(),
        _ => String::new(),
    }
}