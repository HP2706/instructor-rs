
use derive_builder::Builder;
use async_openai::types::CreateChatCompletionRequest;

fn main() {
    let mut x = CreateChatCompletionRequest::default();

    if x.stream == Some(true) {
        println!("stream");
    } else {
        println!("not stream");
    }

    x.stream = Some(true);
    if x.stream == Some(true) {
        println!("stream");
    } else {
        println!("not stream");
    }


    println!("{:?}", x);
}
