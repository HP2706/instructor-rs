use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use validator::Validate;
use std::fmt::Debug; // Add this import to use Debug

pub trait MetaTrait<T> where T: JsonSchema + Serialize + Debug + Default + Validate + for<'de> Deserialize<'de> + Clone {}

impl<T> MetaTrait<T> for T
where T: JsonSchema + serde::Serialize + Debug + Default + Validate + for<'de> Deserialize<'de> + Clone {}
