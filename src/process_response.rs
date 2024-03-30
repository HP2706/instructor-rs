use validator::{ValidateArgs, ValidationErrors};

use openai_api_rs::v1::chat_completion::ChatCompletionResponse;
use serde::{Serialize, Deserialize};
use crate::mode::Mode;
use crate::{enums::IterableOrSingle};
use openai_api_rs::v1::chat_completion::{
    ChatCompletionRequest, ChatCompletionMessage, 
    MessageRole, Content
};
use crate::traits::OpenAISchema;
use crate::enums::Error;
use std::collections::HashMap;
use crate::enums::InstructorResponse;


pub fn handle_response_model<T>(
    response_model: Option<IterableOrSingle<T>>, 
    mode: Mode, 
    kwargs : ChatCompletionRequest
) -> Result<(Option<IterableOrSingle<T>>, ChatCompletionRequest), Error> 
where
    T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchema<T> + schemars::JsonSchema
{
    let mut new_kwargs = kwargs.clone();
    match response_model.as_ref()  {
        Some(Iterable_model) => {
            let schema =  match Iterable_model {
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
        },
        _ => {
            return Ok((response_model, kwargs));
        }
    }
}

pub fn process_response<T>(
    response: &ChatCompletionResponse,
    response_model : &Option<IterableOrSingle<T>>,
    stream: bool,
    validation_context: &<T as OpenAISchema<T>>::Args,
    mode: Mode,
) -> Result<InstructorResponse<T>, Error>
where
    T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchema<T, Args = T> ,
{   

    /* if response_model.is_none() {
        return Ok(InstructorResponse::Completion(response));
    }
    */

    let response_model = response_model.as_ref().unwrap();
    let res : Result<T, Error> = T::from_response(response, validation_context, mode);
    let model = match res {
        Ok(model) => model,
        Err(e) => {
            return Err(e);
        }
    };

    Err(Error::NotImplementedError("This feature is not yet implemented.".to_string()))
}
