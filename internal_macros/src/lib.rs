extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(SchemaToString)]
pub fn schema_to_string_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let gen = quote! {
        impl #name {
            pub fn schema_to_string() -> String {
                let schema = schemars::schema_for!(#name);
                serde_json::to_string_pretty(&schema).unwrap()
            }
        }

        // Implement the DumpSchema trait for the type
        impl shared::DumpSchema for #name {
            fn schema_to_string() -> String {
                let schema = schemars::schema_for!(#name);
                serde_json::to_string_pretty(&schema).unwrap()
            }
        }
    };
    gen.into()
}