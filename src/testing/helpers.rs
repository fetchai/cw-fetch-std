use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    to_json_binary, Addr, Empty, Env, OwnedDeps, SystemError, SystemResult, Uint256, WasmQuery,
};
use cosmwasm_std::{BankMsg, Coin, ContractResult as StdContractResult, Response, SubMsg};

pub fn deps_with_creator(
    creator: Addr,
    contract_address: Addr,
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();
    let mut querier = MockQuerier::default();

    // clone values into the closure
    let creator_for_closure = creator.clone();
    let contract_address_for_closure = contract_address.clone();

    querier.update_wasm(move |request| match request {
        WasmQuery::ContractInfo { contract_addr } => {
            if contract_addr == contract_address_for_closure.as_str() {
                // Return JSON directly instead of constructing ContractInfoResponse (non_exhaustive)
                let payload = serde_json::json!({
                    "code_id": 0u64,
                    "creator": creator_for_closure.as_str(),
                    "admin": creator_for_closure.as_str(),
                    "pinned": false,
                    "ibc_port": null,
                    "ibc2_port": null
                });

                SystemResult::Ok(StdContractResult::Ok(to_json_binary(&payload).unwrap()))
            } else {
                SystemResult::Err(SystemError::NoSuchContract {
                    addr: contract_addr.clone(),
                })
            }
        }
        _ => panic!("unexpected wasm query: {request:?}"),
    });

    deps.querier = querier;
    deps
}
pub fn assert_err<T, E: std::fmt::Debug>(result: &Result<T, E>, error: &E) {
    // Check if result contains specific error
    match result {
        Ok(_) => panic!("Expected Err, got Ok"),
        Err(res_error) => assert_eq!(format!("{:?}", res_error), format!("{:?}", error)),
    }
}

pub fn assert_transfer(res: &Response, address: &Addr, amount: &u128, denom: &str) {
    let send_msg = SubMsg::new(BankMsg::Send {
        to_address: address.to_string(),
        amount: vec![Coin {
            denom: denom.to_string(),
            amount: Uint256::new(*amount),
        }],
    });

    assert_eq!(res.messages[0], send_msg);
}

pub fn mock_env_with_height(height: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env
}
