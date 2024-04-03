use validator::ValidateArgs;
use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::mode::Mode;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest, ChatCompletionMessage, 
    MessageRole, Content
};
use crate::traits::{OpenAISchema, BaseSchema};
use crate::enums::Error;
use crate::iterable::IterableOrSingle;
use std::collections::HashMap;
use crate::enums::InstructorResponse;
use openai_api_rs::v1::chat_completion::{Tool, ToolType, Function};

pub fn handle_response_model<A, T>(
    response_model: IterableOrSingle<T>, 
    mode: Mode, 
    kwargs : ChatCompletionRequest
) -> Result<(IterableOrSingle<T>, ChatCompletionRequest), Error> 
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    let mut new_kwargs = kwargs.clone();
    
    match mode {
        Mode::TOOLS => {
            new_kwargs.tools = Some(vec![
                Tool {
                    r#type: ToolType::Function,
                    function: T::tool_schema(),
                }
            ]);
        },
        Mode::JSON | Mode::MD_JSON | Mode::JSON_SCHEMA => {
            let schema = match response_model {
                IterableOrSingle::Iterable(_) => {
                    if kwargs.stream == Some(true) {
                        return Err(
                            Error::NotImplementedError("Response model is required for streaming.".to_string())
                        );
                    }
                    
                    format!("Make sure for each schema to return an instance of the JSON, not the schema itself, use commas to seperate the schema/schemas: {:?}", T::openai_schema())
                },
                IterableOrSingle::Single(_) => T::openai_schema(),
            };

            let message = format!(
                "As a genius expert, your task is to understand the content and provide \
                the parsed objects in json that match the following json_schema:\n\n\
                {}\n\n\
                Make sure to return an instance of the JSON, not the schema itself",
                schema
            );
            match mode {
                Mode::JSON => {
                    new_kwargs.response_format = Some(
                        serde_json::to_value(
                            HashMap::from([("type".to_string(), "json_object".to_string())])
                        ).unwrap()
                    );
                },
                Mode::JSON_SCHEMA => {
                    new_kwargs.response_format = Some(
                        serde_json::to_value(
                            HashMap::from(
                                [
                                    ("type".to_string(), "json_object".to_string()),
                                    ("schema".to_string(), schema.to_string())
                                ]
                            )
                        ).unwrap()
                    );
                },
                Mode::MD_JSON => {
                    new_kwargs.messages.push(ChatCompletionMessage {
                        role: MessageRole::user,
                        content: Content::Text(
                            "Return the correct JSON response within a ```json codeblock. not the JSON_SCHEMA".to_string()
                        ),
                        name : None
                    });
                },
                _ => {}
            }

            match new_kwargs.messages.get_mut(0) {
                Some(message) if message.role == MessageRole::system => {
                    if let Content::Text(ref mut text) = message.content {
                        let message_content = format!("\n\n{:?}", text);
                        *text += &message_content; //watch out for bad formatting
                    }
                },
                _ => {
                    new_kwargs.messages.insert(0, ChatCompletionMessage {
                        role: MessageRole::system,
                        content: Content::Text(
                            message
                        ),
                        name: None, // Assuming name is optional and not required here
                    });
                },
            } 
        }  
    }

    return Ok((response_model, new_kwargs));
        
    
}

pub fn process_response<T, A>(
    response: &ChatCompletionResponse,
    response_model : &IterableOrSingle<T>,
    stream: bool,
    validation_context: &A,
    mode: Mode,
) -> Result<InstructorResponse<A, T>, Error>
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{   
    

    /* 
    //TODO
    if response_model is None:
        logger.debug("No response model, returning response as is")
        return response

    if (
        inspect.isclass(response_model)
        and issubclass(response_model, (IterableBase, PartialBase))
        and stream
    ):
        model = response_model.from_streaming_response(
            response,
            mode=mode,
        )
        return model
    */

    return T::from_response(response_model, response, validation_context, mode);
    
}

/* fn extract_response_model<T>(
    response_model: IterableOrSingle<T>, 
    mode: Mode, 
    kwargs : ChatCompletionRequest
) -> Result<>, Error> {
    return Ok(response_model);
}

 */