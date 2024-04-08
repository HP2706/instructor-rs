use validator::ValidateArgs;
use crate::mode::Mode;
use crate::openai_schema::{BaseSchema, BaseArg};
use crate::openai_schema::OpenAISchema;
use crate::dsl::iterable::IterableBase;
use crate::error::Error;
use crate::enums::IterableOrSingle;
use std::collections::HashMap;
use crate::enums::InstructorResponse;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, ChatCompletionResponseFormat, ChatCompletionResponseFormatType, ChatCompletionTool, ChatCompletionToolType, CreateChatCompletionRequest, CreateChatCompletionResponse, Role 
};


use crate::enums::ChatCompletionResponseWrapper;

pub fn handle_response_model<A, T>(
    response_model: IterableOrSingle<T>, 
    mode: Mode, 
    kwargs : &mut CreateChatCompletionRequest
) -> Result<IterableOrSingle<T>, Error>
where
    T: ValidateArgs<'static, Args=A> + BaseSchema,
    A: BaseArg,
{

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
                IterableOrSingle::Single(_) => {
                    if kwargs.stream == Some(true) {
                        return Err(
                            Error::Generic(
                                "stream=True is not supported when using response_model parameter for non-iterables".to_string()
                            )
                        );
                    }
                    
                    format!("Make sure for each schema to return an instance of the JSON, not the schema itself, use commas to seperate the schema/schemas: {:?}", T::openai_schema())
                },
                IterableOrSingle::Iterable(_) => T::openai_schema(),
            };

            let message = format!(
                "As a genius expert, your task is to understand the content and provide
                the parsed objects in JSON that match the following json_schema:\n{}\n
                Make sure to return instances of the JSON, not the schema itself",
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
                    kwargs.messages.push(user_message);
                },
                _ => {}
            }
            
            match &mut kwargs.messages[0] {
                ChatCompletionRequestMessage::System(kwargs_message) => {
                    kwargs_message.content += &message;
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

pub async fn process_response_async<T, A>(
    response: ChatCompletionResponseWrapper,
    response_model : IterableOrSingle<T>,
    validation_context: &A,
    mode: Mode,
) -> Result<InstructorResponse<T>, Error>
where
    T: ValidateArgs<'static, Args=A> + BaseSchema + 'static,
    A: BaseArg + 'static,
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
    match response {
        ChatCompletionResponseWrapper::Stream(res) => {
            println!("streaming response");
            let res = T::from_streaming_response_async(response_model, res, validation_context, mode).await;
            Ok(res)
        }
        ChatCompletionResponseWrapper::Single(res) => {
            return T::from_response(&response_model, &res, validation_context, mode);
        }
    }
    
}

/* fn extract_response_model<T>(
    response_model: IterableOrSingle<T>, 
    mode: Mode, 
    kwargs : ChatCompletionRequest
) -> Result<>, Error> {
    return Ok(response_model);
}

 */