use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

// This code is based on https://github.com/CosmWasm/cosmwasm/blob/v1.5.3/packages/schema-derive/src/query_responses.rs

#[proc_macro_derive(QueryResponsesHybrid, attributes(returns, nested))]
pub fn derive_query_responses_hybrid(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(
            enum_name,
            "QueryResponsesHybrid can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    let mut variant_handlers = Vec::new();

    for variant in data_enum.variants {
        let variant_name = variant.ident;
        let variant_str = to_snake_case(&variant_name.to_string());

        let returns_attr = variant.attrs.iter().find(|a| a.path().is_ident("returns"));
        let is_nested = variant.attrs.iter().any(|a| a.path().is_ident("nested"));

        if let Some(attr) = returns_attr {
            // #[returns(Type)]
            let meta = attr
                .parse_args::<syn::Type>()
                .expect("Failed to parse #[returns(Type)]");
            variant_handlers.push(quote! {
                map.insert(#variant_str.to_string(), ::cosmwasm_schema::schema_for!(#meta));
            });
        } else if is_nested {
            let Fields::Unnamed(fields) = variant.fields else {
                return syn::Error::new_spanned(
                    variant_name,
                    "#[nested] variant must be tuple-like",
                )
                .to_compile_error()
                .into();
            };
            if fields.unnamed.len() != 1 {
                return syn::Error::new_spanned(
                    variant_name,
                    "#[nested] variant must have exactly one field",
                )
                .to_compile_error()
                .into();
            }
            let nested_type = &fields.unnamed.first().unwrap().ty;

            variant_handlers.push(quote! {
                for (k, v) in <#nested_type as ::cosmwasm_schema::QueryResponses>::response_schemas_impl() {
                    if map.insert(k.clone(), v).is_some() {
                        panic!("duplicate query name '{}'", k);
                    }
                }
            });
        } else {
            return syn::Error::new_spanned(
                variant_name,
                "Missing #[returns(...)] or #[nested] attribute",
            )
            .to_compile_error()
            .into();
        }
    }

    let expanded = quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl ::cosmwasm_schema::QueryResponses for #enum_name {
            fn response_schemas_impl() -> ::std::collections::BTreeMap<String, ::schemars::schema::RootSchema> {
                let mut map = ::std::collections::BTreeMap::new();
                #(#variant_handlers)*
                map
            }
        }
    };

    TokenStream::from(expanded)
}

/// Converts a variant name into the specified casing
fn to_snake_case(input: &str) -> String {
    // this was stolen from serde for consistent behavior
    let mut snake = String::new();
    for (i, ch) in input.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}
