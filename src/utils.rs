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
pub fn Load_And_Validate<'v_a, T>(json: &'v_a str, args: T::Args) -> Result<T, ValidationErrors>
where
    T: ValidateArgs<'v_a> + Serialize + Deserialize<'v_a>,
{
    let data: T = serde_json::from_str::<T>(json).unwrap();
    match data.validate_args(args) {
        Ok(_) => Ok(data),
        Err(e) => Err(e),
    }
}

pub fn is_async(func: &ItemFn) -> bool {
    func.sig.asyncness.is_some()
}
