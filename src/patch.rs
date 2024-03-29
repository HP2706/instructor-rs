
use openai_api_rs::v1::error::APIError as OpenAIError;

fn retry_sync<F, R>(func: F, max_retries: i32) -> Result<R, OpenAIError>
where
    F: Fn() -> Result<R, OpenAIError>,
{
    let mut attempts = 0;
    loop {
        match func() {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_retries => attempts += 1,
            Err(e) => return Err(e),
        }
    }
}

