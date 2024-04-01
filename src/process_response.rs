use validator::ValidateArgs;


use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use crate::mode::Mode;

use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest, ChatCompletionMessage, 
    MessageRole, Content
};
use crate::traits::{OpenAISchema, BaseSchema};
use crate::enums::{Error, IterableOrSingle};
use std::collections::HashMap;
use crate::enums::InstructorResponse;

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
    let schema = match response_model {
        IterableOrSingle::Iterable(_) => {
            if kwargs.stream == Some(true) {
                return Err(
                    Error::NotImplementedError("Response model is required for streaming.".to_string())
                );
            }
            T::openai_schema()
            
        },
        IterableOrSingle::Single(_) => T::openai_schema(),
    };

    match mode {
        Mode::FUNCTIONS => {
            //TODO
            //from instructor python
            // new_kwargs["functions"] = [response_model.openai_schema]  # type: ignore
            // new_kwargs["function_call"] = {"name": response_model.openai_schema["name"]}  # type: ignore
        },
        Mode::TOOLS | Mode::MISTRAL_TOOLS => {
            //let func = Function { Function::new(response_model.openai_schema.name.clone()) };
            //let tool = Tool { ToolType::Function, response_model.openai_schema.name.clone() };
            //kwargs.tools(vec![tool])  
        },
        Mode::JSON | Mode::MD_JSON | Mode::JSON_SCHEMA => {
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

    match response_model {
        IterableOrSingle::Single(model) => {
            let model = process_one_response(response, model, stream, validation_context, mode);
            match model {
                Ok(model) => {
                    return Ok(InstructorResponse::One(model));
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        IterableOrSingle::Iterable(model) => {
            return Err(
                Error::NotImplementedError("Response model for iterable is not implemnted.".to_string())
            );
        }
    }
    
}

fn process_one_response<T, A>(
    response: &ChatCompletionResponse,
    response_model : &T,
    stream: bool,
    validation_context: &A,
    mode: Mode,
) -> Result<T, Error>
where
    T: ValidateArgs<'static, Args=A> + BaseSchema<T>,
    A: 'static + Copy,
{
    let res : Result<T, Error> = T::from_response(response_model, response, validation_context, mode);
    return match res {
        Ok(model) => {
            println!("process_response result: {:?}", model);
            Ok(model)
        },
        Err(e) => {
            return Err(e);
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