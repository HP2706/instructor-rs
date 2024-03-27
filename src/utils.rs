use validator::{ValidateArgs, ValidationErrors};

pub fn validate_with_args<'v_a, T>(value: T, args: T::Args) -> Result<T, ValidationErrors>
where
    T: ValidateArgs<'v_a>,
{
    match value.validate_args(args) {
        Ok(_) => Ok(value),
        Err(e) => Err(e),
    }
}

