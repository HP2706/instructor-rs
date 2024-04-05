use validator::ValidateArgs;
use crate::mode::Mode;
use crate::traits::{OpenAISchema, BaseSchema};
use crate::enums::Error;
use crate::iterable::IterableOrSingle;
use std::collections::HashMap;
use crate::enums::InstructorResponse;
use openai_api_rs::v1::chat_completion::{Tool, ToolType, Function};

pub fn handle_response_model<A, T>(
    response_model: IterableOrSingle<T>, 
    mode: Mode, 
    kwargs : &mut CreateChatCompletionRequest
) -> Result<IterableOrSingle<T>, Error>
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{


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
        Mode::TOOLS => {
            kwargs.tools = Some(
                vec![
                ChatCompletionTool {
                    r#type: ChatCompletionToolType::Function,
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
                    kwargs.response_format = Some(
                        ChatCompletionResponseFormat {
                            r#type: ChatCompletionResponseFormatType::JsonObject,
                        }
                    );
                },
                Mode::JSON_SCHEMA => {
                    kwargs.response_format = Some(
                        ChatCompletionResponseFormat {
                            r#type: ChatCompletionResponseFormatType::JsonObject,
                        }
                    );
                    //no schema specified....  ("schema".to_string(), schema.to_string())
                },
                Mode::MD_JSON => {
                    let user_message = ChatCompletionRequestMessage::User(
                        ChatCompletionRequestUserMessage {
                            content: ChatCompletionRequestUserMessageContent::Text(
                                "Return the correct JSON response within a ```json codeblock. not the JSON_SCHEMA".to_string()
                            ),
                            role: Role::User,
                            name: None,
                        }
                    );
                },
                _ => {}
            }
            
            match &kwargs.messages[0] {
                ChatCompletionRequestMessage::System(_) => {
                    //TODO
                }
                _=> {
                    kwargs.messages.insert(0, 
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                            role: Role::System,
                            content: message,
                            name: None, // Assuming name is optional and not required here
                        }
                    ));
                }
            }
        }  
    }

    return Ok(response_model);
        
    
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