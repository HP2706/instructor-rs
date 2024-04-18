This is a rust port of the [Instructor](https://github.com/jxnl/instructor/tree/ba93407b050bcbfbf5716cc67856f8491a00b98a) library

the library is built on top of the most popular openai rust client: [async_openai](https://github.com/64bit/async-openai)
this library is inherently async in nature, however it is possible to make this run in non-async function by using the [tokio runtime](https://tokio.rs/tokio/topics/bridging).

```rust
fn main(){
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            //your async code
        })
}
```

by using block_on, we can call async function in synchronous functions.

##Features
- Current features:
  - [x] openai support
  - [x] async streaming
  - [x] async non-streaming
  - [x] automatic retry logic
  - [x] custom struct validation
  - [x] support for Together api
  - [x] support for ollama

##Lacking
- missing features:
  - [ ] anthropic support
  - [ ] synchronous support(you can try to use tokio::block_on to make it work crudely)
  - [ ] advanced validation( validation conditioned on multiple fields at once)
  - [ ] support for things like Union[datamodel1, datamodel2] 

##Installation guide
To get started, make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

copy the following to your Cargo.toml

instructor-rs = { git = "https://github.com/HP2706/instructor-rs"}

use in rust with 
```rust
use instructor_rs::patch::Patch;
use instructor_rs::mode::Mode;
use async_openai::Client;
```

##Concepts

The concepts are very similar to that of instructor. The biggest difference being how class/struct validation works.
in instructor you would define a pydantic model

```python
from pydantic import BaseModel, Field, field_validator

class Add(BaseModel):
    '''add the two numbers a and b must each be positive and larger than a number''' 
    #this string is actually captured in instructor
    a : int = Field(..., description="a must be positive")
    b : int = Field(..., description="b must be positive")
    @field_validator("a")
    def a_must_be_positive(cls, v):
        if v <= 0:
            raise ValueError("a must be positive")
        return v
```

pydantic takes care of serialization/deserialization and validation.

In rust there is no unified library for doing these things and thus the way we define our classes is a bit different. 
We combine 3 different libraries to achieve what pydantic does.

1. Serde for serialization/deserialization 
2. Schemars to generate json schema and annotate with comments (think Field(..., description=""))
3. Validators for struct validation 

concretely this will look something like this:
```rust
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use validator::{Validate, ValidationError};

#[derive(JsonSchema, Serialize, Debug, Default, Validate, Deserialize, Clone)]
#[schemars(description="add the two numbers a and b must each be positive and larger than a number c=10")]
struct Add {
    #[schemars(description="a must be positive")]
    #[validate(range(min = 0))] // these are built in validators
    #[validate(custom(function = "a_geq_c", arg = "&'v_a i64"))]
    a : i64,
    #[schemars(description="a must be positive")]
    #[validate(range(min = 0))] // these are built in validators
    #[validate(custom(function = "a_geq_c", arg = "&'v_a i64"))]
    b : i64,
}

fn a_geq_c(a: i64, c: &i64) -> Result<(), validator::ValidationError> {
    if a < *c {
        let err_msg = format!("a must be greater than or equal to {}", c);
        return Err(ValidationError::new(&*Box::leak(err_msg.into_boxed_str())));
    }
    Ok(())
}
```
pydantic offer a lot more flexibility in how validation should work, for instance doing validation you can condition your validation in multiple fields and determine ordering of validation. these things are not implemented in this library. 

it is also important to note that nested custom validation does not work with the validators crate. Thus if you have fields that themselves implement the Validate trait the behaviourt might be [unanticipated](https://github.com/Keats/validator?tab=readme-ov-file).

##providers

the async_openai allows some customizability in the client, which means that you can use openai-api compatible endpoints.

for instance you can use the Together_ai endpoint chat completions endpoint like this:

```rust
use async_openai::config::OpenAIConfig;
use std::env;
let api_key = env::var("TOGETHER_API_KEY").unwrap();
let endpoint = "https://api.together.xyz/v1";

    // Create an OpenAIConfig with the specified API key and endpoint
    let config = OpenAIConfig::default()
    .with_api_key(api_key)
    .with_api_base(endpoint.to_string());

// Create a Client with the specified configuration
let client = Client::with_config(config);
let patched_client = Patch { client: client, mode: Some(Mode::TOOLS) };
```

you can use local models via ollama

```rust
//GROQ_API_KEY
let api_key = "ollama"; //this api key will not get used;
let endpoint ="http://localhost:11434/v1";

// Create an OpenAIConfig with the specified API key and endpoint
let config = OpenAIConfig::default()
.with_api_key(api_key)
.with_api_base(endpoint.to_string());

// Create a Client with the specified configuration
let client = Client::with_config(config);
let mode = Mode::TOOLS;
let patched_client = Patch { client: client.clone(), mode: Some(mode) };
let model = "mistral:latest";
```


##examples 

all examples assume the following is imported

```rust
use schemars::JsonSchema;
use std::vec;
use instructor_rs::mode::Mode;  
use instructor_rs::patch::Patch;
use instructor_rs::enums::IterableOrSingle;
use model_traits_macro::derive_all;
use serde::{Deserialize, Serialize};
use validator::Validate;
use instructor_rs::common::GPT4_TURBO_PREVIEW;
use async_openai::types::{
    CreateChatCompletionRequestArgs,
    ChatCompletionRequestUserMessage, ChatCompletionRequestMessage, Role,
    ChatCompletionRequestUserMessageContent
};
use async_openai::Client;
use instructor_rs::enums::InstructorResponse;
use futures::stream::StreamExt;
```

lets starte with a basic example

```rust 

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::TOOLS) };

    #[derive(JsonSchema, Serialize, Debug, Default, Deserialize, Clone)] 
    ///we cannot use #[derive_all] here as enums cannot derive Validate Trait
    enum TestEnum {
        #[default]
        PM,
        AM,
    }

    ///we use rust macros to derive traits to reduce boilerplate, however this reduces visibility, you can us both
    ///[derive_all] basically inserts: #[derive(
    ///  JsonSchema, Serialize, Debug, Default, 
    ///  Validate, Deserialize, Clone 
    ///)] remember that you still have to import the traits 
    #[derive_all]
    #[schemars(description = "this is a description of the weather api")]
    struct Weather {
        //#[schemars(description = "am or pm")]
        //time_of_day: TestEnum,
        #[schemars(description = "this is the hour from 1-12")]
        time: i64,
        city: String,
    }
    
    let req = CreateChatCompletionRequestArgs::default()
    .model(GPT4_TURBO_PREVIEW.to_string())
    .messages(vec![
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage{
                role: Role::User,
                content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                what is the weather at 10 in the evening in new york? 
                and what is the whether in the biggest city in Denmark in the evening?
                ")),
                name: None,
            }
        )],
    ).build().unwrap();

    let result = patched_client.chat_completion(
        ///we wrap our model in an Iterable enum to allow more than one function call 
        /// a bit like Iterable[BaseModel] in instructor
        ///we use default to produce a default instance of the struct(this is never used itself, but a walkaround rust
        /// not allowing struct types as function arguments)
        IterableOrSingle::Iterable(Weather::default()), 
        (), // the validation function
        1, // max_retries
        req, // our openai request
    );

    println!("result: {:?}", result.await);
    ///Ok(Many([Weather { time: 10, city: "New York" }, Weather { time: 10, city: "Copenhagen" }]))
    Ok(())
}```


```rust

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    #[derive_all]    
    struct Number {
        #[schemars(description = "the value")]
        value: i64,
    }
    
    let req = CreateChatCompletionRequestArgs::default()
    .model(GPT4_TURBO_PREVIEW.to_string())
    .messages(vec![
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage{
                role: Role::User,
                content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                write 2 numbers in the specified json format
                ")),
                name: None,
            }
        )],
    )
    .stream(true)
    .model(GPT4_TURBO_PREVIEW.to_string())
    .build()
    .unwrap();

    let result = patched_client.chat_completion(
        IterableOrSingle::Iterable(Number::default()),
        (), // the validation function
        1, // max_retries
        req, // our openai request
    );

    use std::time::Instant;


    let model = result.await.unwrap(); // we accept panic when using unwrap()
    match model {
        InstructorResponse::Many(x) => println!("result: {:?}", x),
        InstructorResponse::One(x) => println!("result: {:?}", x),
        InstructorResponse::Stream(mut x) => {
            let t0 = Instant::now();
            while let Some(x) = x.next().await {
                println!("model: {:?} at time {:?}", x, t0.elapsed());
            }
        },
    }
    /// model: Number { value: 1 } at time 1.1
    /// model: Number { value: 2 } at time 1,8
    Ok(())
}
```

lets do a more complex example, that relies on custom validation and serialization/deserialization

```rust

#[derive_all]
///we use rust macros to derive certain traits in order to serialize/deserialize format as json and Validate
///#[derive(
///  JsonSchema, Serialize, Debug, Default, 
///  Validate, Deserialize, Clone 
///)]
struct Director {
    ///We annotate the fields with the description of the field like you would do Field(..., description = "...") in pydantic
    #[schemars(description = "A string value representing the name of the person")]
    name : String,
    
    #[schemars(description = "The age of the director, the age of the director must be a multiple of 3")]
    #[validate(custom(function = "check_is_multiple", arg = "i64"))]
    ///we define custom validation function that can take in foreign input and perform validation logic based on input
    age : i64,
    #[schemars(description = "year of birth")] 
    birth_year : i64
}  

fn check_is_multiple(age: i64, arg : i64) -> Result<(), ValidationError> {
    if age % 3 == 0 {
        Ok(())
    } else {
        Err(ValidationError::new("The age {} is not a multiple of 3"))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();
    let patched_client = Patch { client, mode: Some(Mode::JSON) };

    let req = CreateChatCompletionRequestArgs::default()
    .model(GPT4_TURBO_PREVIEW.to_string())
    .messages(vec![
        ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage{
                role: Role::User,
                content:    ChatCompletionRequestUserMessageContent::Text(String::from("
                return an instance of an director that is more than 60 years old (hint steven spielberg)
                ")),
                name: None,
            }
        )],
    ).build().unwrap();

    ///we wrap in an Iterable enum to allow more than one function call 
    /// a bit like List[Type[BaseModel]] or Iterable[Type[BaseModel]] in instructor
    let result = patched_client.chat_completion(
        IterableOrSingle::Single(Director::default()),
        (2024-60),
        2,
        req,
    );

    println!("result: {:?}", result.await);
    /// Ok(InstructorResponse::Single({ name: "Steven Spielberg", age: 77, birth_year: 1946 }))
    Ok(())
}
```