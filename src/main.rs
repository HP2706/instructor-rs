use model_traits_macro::derive_all;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::api::Client;
use schemars::JsonSchema;
use std::{env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::iterable::IterableOrSingle;
use serde::{Deserialize, Serialize};
use validator::Validate;
use openai_api_rs::v1::common::{GPT4_TURBO_PREVIEW, GPT3_5_TURBO};
use instructor_rs::streaming::{process_streaming_response_v2, ChatCompletionResponseWrapper};
use instructor_rs::utils::extract_json_from_stream;

fn call_stream(req : &ChatCompletionRequest) -> ChatCompletionResponseWrapper {
    let url = "https://api.openai.com/v1/chat/completions";
    let api_key = env::var("OPENAI_API_KEY").unwrap().to_string();
    let mut out = String::new();
    let oa_client = Client::new(api_key);
    let oa_req = oa_client.build_request(minreq::post(url), false);
    let mut resp = oa_req.with_json(&req).unwrap().send_lazy(); // Correctly handle the Result here
    process_streaming_response_v2(resp.unwrap())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //func_call();
    use async_openai::{
        types::{CreateImageRequestArgs, ImageSize, ResponseFormat},
        Client,
    };
    use std::error::Error;
    let client = Client::new();

    let request = CreateImageRequestArgs::default()
        .prompt("cats on sofa and carpet in living room")
        .n(2)
        .response_format(ResponseFormat::Url)
        .size(ImageSize::S256x256)
        .user("async-openai")
        .build()?;

    let response = client.images().create(request).await?;
    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            return an essays of 200 words
            ")),
            name: None,
        }],
    ).stream(true);

    #[derive_all]
    struct Actor {
        ///We annotate the fields with the description of the field like you would do Field(..., description = "...") in pydantic
        #[schemars(description = "A string value representing the name of the person")]
        name : String,
        #[schemars(description = "The age of the actor")]
        age : i64,
        #[schemars(description = "3 movies the actor has been associated with")]
        
        ///we use the validate macros to validate specific fields 
        ///here we check that the movies vector has exactly 3 items
        #[validate(length(min = 3, max = 3, message = "movies must contain exactly 3 items"))]
        movies : Vec<String>,
    }  

    let client = Client::new().unwrap();
    let patched_client = Patch { client, mode: Some(Mode::JSON) };
    
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Actor::default()),
        (),
        1,
        true, //consider removing this from the api, it appears streaming is not supported
        req,
    );

    Ok(())
}


