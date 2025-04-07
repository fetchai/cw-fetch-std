use cosmwasm_schema::{cw_serde, QueryResponses};
use query_responses_hybrid_derive::QueryResponsesHybrid;
use schemars::schema::RootSchema;
use std::collections::BTreeMap;

#[cw_serde]
#[derive(QueryResponses)]
enum SubQuery {
    #[returns(FooResponse)]
    Foo {},
    #[returns(BarResponse)]
    Bar {},
}

#[cw_serde]
struct FooResponse {
    pub value: String,
}

#[cw_serde]
struct BarResponse {
    pub number: u64,
}

#[cw_serde]
#[derive(QueryResponsesHybrid)]
enum MyQuery {
    #[returns(MyResponse)]
    MyField { name: String },

    #[nested]
    Sub(SubQuery),
}

#[cw_serde]
struct MyResponse {
    pub message: String,
}

#[test]
fn hybrid_derive_works_and_respects_snake_case() {
    let map: BTreeMap<String, RootSchema> = <MyQuery as QueryResponses>::response_schemas_impl();

    let keys: Vec<String> = map.keys().cloned().collect();
    assert!(keys.contains(&"my_field".to_string()));
    assert!(keys.contains(&"foo".to_string()));
    assert!(keys.contains(&"bar".to_string()));
    assert!(!keys.contains(&"MyField".to_string()));
    assert!(!keys.contains(&"Foo".to_string()));
}
