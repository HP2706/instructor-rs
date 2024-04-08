use validator::ValidateArgs;
use crate::mode::Mode;
use crate::openai_schema::{BaseSchema, BaseArg};
use crate::openai_schema::OpenAISchema;
use crate::dsl::iterable::IterableBase;
use crate::error::Error;
use crate::enums::IterableOrSingle;
use crate::enums::InstructorResponse;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage, 
    ChatCompletionRequestUserMessageContent, ChatCompletionResponseFormat, ChatCompletionResponseFormatType, 
    ChatCompletionTool, ChatCompletionToolType, CreateChatCompletionRequest, Role 
};
use crate::enums::ChatCompletionResponseWrapper;

/// this function ads a prompt to the request messages or to the tools field(preferred) 
/// 
/// # Arguments
/// * `response_model`: `&IterableOrSingle<T>` - a reference to an enum wrapper that is very similar to Iterable[model] in instructor
///   Can be either a single instance or an iterable collection of instances, depending on the use case.
/// * `mode`: `Mode` - the mode to use for processing the response
/// * `kwargs`: `&mut CreateChatCompletionRequest` - a mutable reference to a request object to modify
pub fn handle_response_model<A, T>(
    response_model: &IterableOrSingle<T>, 
    mode: Mode, 
    kwargs : &mut CreateChatCompletionRequest
) -> Result<(), Error>
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
    Ok(())
}

/// this function processes the response based on the mode and the response_model and parses the response accordingly. 
/// It supports both streaming outputs and non-streaming outputs.
/// 
/// # Arguments
/// * `response`: `ChatCompletionResponseWrapper` - the response from the OpenAI API
/// * `response_model`: `IterableOrSingle<T>` - the response model to use for processing the response
/// * `validation_context`: `&A` - the validation context to use for processing the response
/// * `mode`: `Mode` - the mode to use for processing the response
/// 
/// # Returns
/// * `Result<InstructorResponse<T>, Error>` - the result of the response processing
/// 
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
    
    match response {
        ChatCompletionResponseWrapper::Stream(res) => {
            println!("streaming response");
            let res = T::from_streaming_response_async(response_model, res, validation_context, mode).await;
            Ok(res)
        }
        ChatCompletionResponseWrapper::AtOnce(res) => {
            return T::from_response(&response_model, &res, validation_context, mode);
        }
    }
}
