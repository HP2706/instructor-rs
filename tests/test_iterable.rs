use instructor_rs::openai_schema::OpenAISchema;
use schemars::JsonSchema;
use validator::ValidationError;
use std::clone::Clone;
use instructor_rs::enums::IterableOrSingle;
use instructor_rs::utils::{string_to_stream, create_chat_completion_stream};
use instructor_rs::error::Error;
use async_openai::types::CreateChatCompletionResponse;
use tokio;
use validator::Validate;
use serde::{Deserialize, Serialize};
use model_traits_macro::derive_all;
use instructor_rs::dsl::iterable::IterableBase;
use instructor_rs::enums::InstructorResponse;
use futures::stream::TryStreamExt;
use instructor_rs::mode::Mode;

#[derive_all]
struct TestStruct {
    value1: i64,
}     

#[cfg(test)]
mod tests {
    use instructor_rs::dsl::iterable::IterableBase;

    use super::*;

    #[tokio::test]
    async fn tasks_from_chunks_async(){
        
        let values = vec![
            TestStruct { value1: 1 },
            TestStruct { value1: 2 },
            TestStruct { value1: 3 },
        ];

        let text: String = values
        .iter()
        .map(|x| serde_json::to_string(x).unwrap())
        .collect::<Vec<_>>()
        .join(" ");

        let response = TestStruct::tasks_from_chunks_async(
            IterableOrSingle::Iterable(TestStruct::default()),
            string_to_stream(text).await,
            ()
        ).await;

        match response {
            InstructorResponse::Stream(chunks) => {
                let outputs : Vec<TestStruct> = chunks.try_collect().await.unwrap();
                for (output, value) in outputs.iter().zip(values.iter()) {
                    assert_eq!(output.value1, value.value1);
                }

            }
            _ => {
                assert_eq!(true, false);
            }
        
        }

    }

    #[tokio::test]
    async fn test_from_streaming_response_async(){
        let values = vec![
            TestStruct { value1: 1 },
            TestStruct { value1: 2 },
            TestStruct { value1: 3 },
        ];

        let text: String = values
        .iter()
        .map(|x| serde_json::to_string(x).unwrap())
        .collect::<Vec<_>>()
        .join(" ");

        let response_stream = create_chat_completion_stream(string_to_stream(text).await);
        
        let response = TestStruct::from_streaming_response_async(
            IterableOrSingle::Iterable(TestStruct::default()),
            response_stream.await,
            &(),
            Mode::JSON
        ).await;

        match response {
            InstructorResponse::Stream(chunks) => {
                let outputs : Vec<TestStruct> = chunks.try_collect().await.unwrap();
                for (output, value) in outputs.iter().zip(values.iter()) {
                    println!("output: {:?}", output);
                    assert_eq!(output.value1, value.value1);
                }
            }
            _ => {
                println!("failed");
                assert_eq!(true, false);
            }
        }

    }
}