use schemars::JsonSchema;
use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use std::{default, env, vec};
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use validator::Validate;
use instructor_rs::iterable::IterableOrSingle;
use openai_api_rs::v1::common::GPT4_TURBO_PREVIEW;
use serde::{Serialize, Deserialize};



fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new(env::var("OPENAI_API_KEY").unwrap().to_string());
    let patched_client = Patch { client, mode: Some(Mode::TOOLS) };

    #[derive(JsonSchema, Serialize, Debug, Default, Deserialize, Clone)]
    enum TestEnum {
        #[default]
        PM,
        AM,
    }

    #[derive(
        JsonSchema, Serialize, Debug, Default, 
        Validate, Deserialize, Clone
    )]
    #[schemars(description = "this is a description of the weather api")]
    struct Weather {
        //#[schemars(description = "am or pm")]
        //time_of_day: TestEnum,
        #[schemars(description = "this is the hour from 1-12")]
        time: i64,
        city: String,
    }
    
 
    ///our request
    let req = ChatCompletionRequest::new(
        GPT4_TURBO_PREVIEW.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: chat_completion::Content::Text(String::from("
            what is the weather at 10 in the evening in new york? 
            and what is the whether in the biggest city in Denmark in the evening?
            ")),
            name: None,
        }],
    );

    let result = patched_client.chat_completion(
        ///we wrap in an Iterable enum to allow more than one function call 
        /// a bit like List[Type[BaseModel]] or Iterable[Type[BaseModel]] in instructor
        IterableOrSingle::Iterable(Weather::default()),
        (),
        1,
        false, //consider removing this from the api, it appears streaming is not supported
        req,
    );

    println!("result: {:?}", result);
    ///Ok(Many([Weather { time: 10, city: "New York" }, Weather { time: 10, city: "Copenhagen" }]))
    
    Ok(())
}


