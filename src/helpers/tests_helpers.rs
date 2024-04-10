use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::ContractResult as StdContractResult;
use cosmwasm_std::{
    to_json_binary, Addr, ContractInfoResponse, Empty, OwnedDeps, SystemError, SystemResult,
    WasmQuery,
};

pub fn deps_with_creator(
    creator: Addr,
    contract_address: Addr,
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();
    let mut querier = MockQuerier::default();
    querier.update_wasm(move |request| match request {
        WasmQuery::ContractInfo { contract_addr } => {
            if *contract_addr == contract_address {
                let mut response = ContractInfoResponse::default();
                response.admin = Some(creator.to_string());
                SystemResult::Ok(StdContractResult::Ok(to_json_binary(&response).unwrap()))
            } else {
                SystemResult::Err(SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                })
            }
        }

        _ => {
            panic!()
        }
    });
    deps.querier = querier;
    deps
}
