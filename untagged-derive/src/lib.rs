extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};
extern crate serde_cw_value;
extern crate serde;

#[proc_macro_derive(GenericUntaggedEnum)]
pub fn derive_generic_untagged_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;

    let Data::Enum(data_enum) = &input.data else {
        panic!("#[derive(GenericUntaggedEnum)] can only be used on enums");
    };

    let deserializers = data_enum.variants.iter().map(|v| {
        let variant = &v.ident;

        let field_ty = match &v.fields {
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                &fields.unnamed.first().unwrap().ty
            }
            _ => panic!("GenericUntaggedEnum only supports tuple enum variants with one field."),
        };

        quote! {
            if let ::serde::__private::Ok(__ok) = ::serde::__private::Result::map(
                <#field_ty as ::serde::Deserialize>::deserialize(
                    ::serde_cw_value::ValueDeserializer::<::serde_cw_value::DeserializerError>::new(__content.clone())
                ),
                #enum_name::#variant,
            ) {
                return ::serde::__private::Ok(__ok);
            }
        }
    });

    let output = quote! {
        #[automatically_derived]
        impl<'de> ::serde::Deserialize<'de> for #enum_name {
            fn deserialize<__D>(__deserializer: __D) -> ::serde::__private::Result<Self, __D::Error>
            where
                __D: ::serde::Deserializer<'de>,
            {
                let __content = match <::serde_cw_value::Value>::deserialize(__deserializer) {
                    ::serde::__private::Ok(__val) => __val,
                    ::serde::__private::Err(__err) => return serde::__private::Err(__err),
                };

                #(#deserializers)*

                serde::__private::Err(serde::de::Error::custom(
                    "data did not match any variant of untagged enum"
                ))
            }
        }
    };

    TokenStream::from(output)
}
