use crate::types::JsonError;
use serde::Serialize;
use serde_json::Error as SerdeJsonError; 
pub trait LoadFromJson<T> {
    fn load_from_json(json: &str) -> Result<T, SerdeJsonError>
    where
        T: Serialize;
}

