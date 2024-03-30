extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use shared::OpenAISchema;

#[proc_macro_derive(OpenAISchema)]
pub fn schema_to_string_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let gen = quote! {
        impl OpenAISchema for #name {
            fn openai_schema() -> String {
                let schema = schemars::schema_for!(#name);
                serde_json::to_string_pretty(&schema).unwrap()
            }

            fn model_validate_json<T>(
                data: &str,
                validation_context: T::Args,
            ) -> Result<T, Error>
            where
                T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de>,
            {
                match serde_json::from_str::<T>(data) {
                    Ok(data) => {
                        match data.validate_args(validation_context) {
                            Ok(_) => Ok(data),
                            Err(e) => Err(Error::Validation(e)),
                        }
                    }
                    Err(e) => Err(Error::Validation(ValidationErrors::new())),
                }
            }

            fn from_response<T>(
                response: &ChatCompletionResponse,
                validation_context: T::Args,
                mode: Mode,
            ) -> Result<T, Error>
            where
                T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchema,
            {
                match mode {
                    Mode::JSON | Mode::JSON_SCHEMA | Mode::MD_JSON => {
                        Self::parse_json(response, validation_context)
                    }
                    _ => {
                        return Err(
                            Error::NotImplementedError("This feature is not yet implemented.".to_string())
                        );   
                    }
                }
            }

            fn parse_json<T>(
                completion : &ChatCompletionResponse,
                validation_context: T::Args,
            ) -> Result<T, Error>
            where
                T: ValidateArgs<'static> + Serialize + for<'de> Deserialize<'de> + OpenAISchema,
            {
                let text = completion.choices[0].message.content.clone().unwrap();
                let json_extract = extract_json_from_codeblock(&text);
                T::model_validate_json(&json_extract, validation_context)
            }

        }
    };
    gen.into()
}