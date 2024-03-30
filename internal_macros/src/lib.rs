extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};



#[proc_macro_derive(OpenAISchema)]
pub fn schema_to_string_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let gen = quote! {
        impl shared::OpenAISchemaSpec for #name {
            fn openai_schema() -> String {
                let schema = schemars::schema_for!(#name);
                serde_json::to_string_pretty(&schema).unwrap()
            }

            fn model_validate_json<T>(
                data: &str,
                validation_context: T::Args,
            ) -> Result<T, shared::enums::Error>
            where
                T: validator::ValidateArgs<'static> + serde::Serialize + for<'de> serde::Deserialize<'de>,
            {
                match serde_json::from_str::<T>(data) {
                    Ok(data) => {
                        match data.validate_args(validation_context) {
                            Ok(_) => Ok(data),
                            Err(e) => Err(shared::enums::Error::Validation(e)),
                        }
                    }
                    Err(e) => Err(shared::enums::Error::Validation(validator::ValidationErrors::new())),
                }
            }

            fn from_response<T>(
                response: &openai_api_rs::v1::chat_completion::ChatCompletionResponse,
                validation_context: T::Args,
                mode: shared::mode::Mode,
            ) -> Result<T, shared::enums::Error>
            where
                T: validator::ValidateArgs<'static> + serde::Serialize + for<'de> serde::Deserialize<'de> + shared::OpenAISchemaSpec,
            {
                match mode {
                    shared::mode::Mode::JSON | shared::mode::Mode::JSON_SCHEMA | shared::mode::Mode::MD_JSON => {
                        Self::parse_json(response, validation_context)
                    }
                    _ => {
                        return Err(
                            shared::enums::Error::NotImplementedError("This feature is not yet implemented.".to_string())
                        );   
                    }
                }
            }

            fn parse_json<T>(
                completion : &openai_api_rs::v1::chat_completion::ChatCompletionResponse,
                validation_context: T::Args,
            ) -> Result<T,shared::enums::Error>
            where
                T: validator::ValidateArgs<'static> + serde::Serialize + for<'de> serde::Deserialize<'de> + shared::OpenAISchemaSpec,
            {
                let text = completion.choices[0].message.content.clone().unwrap();
                let json_extract = shared::utils::extract_json_from_codeblock(&text);
                T::model_validate_json(&json_extract, validation_context)
            }

        }
    };
    gen.into()
}