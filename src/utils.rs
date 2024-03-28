use validator::{ValidateArgs, Validate, ValidationErrors};
use syn::ItemFn;
use schemars::{JsonSchema, schema_for};
use serde::{Serialize, Deserialize};
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
