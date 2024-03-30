
pub enum Error {
    Validation(validator::ValidationErrors),
    NotImplementedError(String),
    Generic(String),
}
