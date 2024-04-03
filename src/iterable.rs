
use serde::{Deserialize, Serialize};
use validator::{ValidateArgs, ValidationErrors};
use schemars::JsonSchema;


#[derive(Debug, Deserialize, Serialize, Copy, Clone, JsonSchema)]
pub enum IterableOrSingle<T>
where T: ValidateArgs<'static>
{
    Iterable(T), 
    Single(T),
}

impl<T> IterableOrSingle<T>
where T: ValidateArgs<'static>
{
    // This method is now correctly placed outside the ValidateArgs trait impl block
    pub fn unwrap(self) -> T {
        match self {
            IterableOrSingle::Iterable(item) | IterableOrSingle::Single(item) => item,
        }
    }
}

impl<'v_a, T> ValidateArgs<'static> for IterableOrSingle<T>
where
    T: ValidateArgs<'static>,
{
    type Args = T::Args;

    fn validate_args(&self, args: Self::Args) -> Result<(), ValidationErrors> {
        match self {
            IterableOrSingle::Iterable(item) | IterableOrSingle::Single(item) => {
                item.validate_args(args)
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Iterable<T> {
    VecWrapper(Vec<T>),
    // You can add more variants here if you need to wrap T in different iterable types
}

