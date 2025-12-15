use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Expr, Fields, Generics, Type,
    TypeParamBound, Variant,
};

#[proc_macro_derive(QueryResponsesHybrid, attributes(returns, nested))]
pub fn derive_query_responses_hybrid(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident.clone();
    let generics = input.generics.clone();

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(
            enum_name,
            "QueryResponsesHybrid can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    // hardcoded like your old macro; if you want it configurable, swap this to a parsed Path
    let crate_name: syn::Path = parse_quote!(::cosmwasm_schema);

    // For hybrid enums we always "combine subqueries", even for #[returns(...)].
    // That allows mixing #[returns] and #[nested] variants in the same enum.
    let subquery_calls_json: Vec<Expr> = match data_enum
        .variants
        .iter()
        .map(|v| parse_hybrid_variant(&crate_name, v, SchemaBackend::JsonSchema))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let subquery_calls_cw: Vec<Expr> = match data_enum
        .variants
        .iter()
        .map(|v| parse_hybrid_variant(&crate_name, v, SchemaBackend::CwSchema))
        .collect::<syn::Result<Vec<_>>>()
    {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generics handling: match cosmwasm-schema 3.x style
    let (_, type_generics, where_clause) = generics.split_for_impl();
    let impl_generics = impl_generics(
        &crate_name,
        &generics,
        &[parse_quote! { #crate_name::QueryResponses }],
        &[],
    );

    let subquery_len = subquery_calls_json.len();

    let expanded = quote! {
        #[automatically_derived]
        #[cfg(not(target_arch = "wasm32"))]
        impl #impl_generics #crate_name::QueryResponses for #enum_name #type_generics #where_clause {
            fn response_schemas() -> ::std::collections::BTreeMap<String, #crate_name::schemars::schema::RootSchema> {
                let subqueries = [
                    #( #subquery_calls_json, )*
                ];
                #crate_name::combine_subqueries::<#subquery_len, #enum_name #type_generics, _>(subqueries)
            }

            fn response_schemas_cw() -> ::std::collections::BTreeMap<String, #crate_name::cw_schema::Schema> {
                let subqueries = [
                    #( #subquery_calls_cw, )*
                ];
                #crate_name::combine_subqueries::<#subquery_len, #enum_name #type_generics, _>(subqueries)
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(Copy, Clone)]
enum SchemaBackend {
    JsonSchema,
    CwSchema,
}

/// Hybrid variant parser:
/// - #[returns(T)] => returns a single-entry BTreeMap
/// - #[nested] (SubQueryType) => returns SubQueryType::response_schemas*()
fn parse_hybrid_variant(
    crate_name: &syn::Path,
    v: &Variant,
    schema_backend: SchemaBackend,
) -> syn::Result<Expr> {
    let query = to_snake_case(&v.ident.to_string());

    let returns_attr = v.attrs.iter().find(|a| a.path().is_ident("returns"));
    let is_nested = v.attrs.iter().any(|a| a.path().is_ident("nested"));

    match (returns_attr, is_nested) {
        (Some(attr), false) => {
            let response_ty: Type = attr
                .parse_args()
                .map_err(|e| syn::Error::new(e.span(), "return must be a type"))?;

            let schema_expr: Expr = match schema_backend {
                SchemaBackend::JsonSchema => parse_quote!(#crate_name::schema_for!(#response_ty)),
                SchemaBackend::CwSchema => {
                    parse_quote!(#crate_name::cw_schema::schema_of::<#response_ty>())
                }
            };

            // Return a map with single entry for this query
            Ok(parse_quote!({
                let mut m = ::std::collections::BTreeMap::new();
                m.insert(#query.to_string(), #schema_expr);
                m
            }))
        }

        (None, true) => {
            let sub_ty = match &v.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0].ty,
                Fields::Unnamed(fields) => {
                    return Err(syn::Error::new_spanned(
                        fields,
                        "#[nested] variant must have exactly one field",
                    ));
                }
                Fields::Named(_) => {
                    return Err(syn::Error::new_spanned(
                        v,
                        "a struct variant is not a valid subquery",
                    ));
                }
                Fields::Unit => {
                    return Err(syn::Error::new_spanned(
                        v,
                        "a unit variant is not a valid subquery",
                    ));
                }
            };

            let call: Expr = match schema_backend {
                SchemaBackend::JsonSchema => {
                    parse_quote!(<#sub_ty as #crate_name::QueryResponses>::response_schemas())
                }
                SchemaBackend::CwSchema => {
                    parse_quote!(<#sub_ty as #crate_name::QueryResponses>::response_schemas_cw())
                }
            };

            Ok(call)
        }

        (Some(_), true) => Err(syn::Error::new_spanned(
            v,
            "variant cannot have both #[returns(...)] and #[nested]",
        )),

        (None, false) => Err(syn::Error::new_spanned(
            v,
            "missing #[returns(...)] or #[nested] attribute",
        )),
    }
}

/// Takes generics from the type definition and produces generics for the expanded `impl`,
/// adding bounds similarly to cosmwasm-schema 3.x.
///
/// `extra_bounds_on_type_params` are appended to every type parameter unless excluded.
/// (Here we default to none; adapt if you want `#[no_bounds_for(T)]` support.)
fn impl_generics(
    crate_name: &syn::Path,
    generics: &Generics,
    extra_bounds_on_type_params: &[TypeParamBound],
    no_bounds_for: &[syn::Ident],
) -> Generics {
    let mut impl_generics = generics.to_owned();

    for param in impl_generics.type_params_mut() {
        param.default = None;

        if no_bounds_for.iter().any(|id| id == &param.ident) {
            continue;
        }

        param
            .bounds
            .push(parse_quote!(#crate_name::schemars::JsonSchema));
        param
            .bounds
            .push(parse_quote!(#crate_name::cw_schema::Schemaifier));
        param
            .bounds
            .extend(extra_bounds_on_type_params.iter().cloned());
    }

    impl_generics
}

fn to_snake_case(input: &str) -> String {
    let mut snake = String::new();
    for (i, ch) in input.char_indices() {
        if i > 0 && ch.is_uppercase() {
            snake.push('_');
        }
        snake.push(ch.to_ascii_lowercase());
    }
    snake
}
