use schemars::{JsonSchema, schema_for};
use serde::Serialize;

pub trait DumpSchema {
    fn schema_to_string(&self) -> String;
}

