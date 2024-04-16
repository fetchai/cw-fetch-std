use cosmwasm_std::testing::{mock_dependencies, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    to_json_binary, Addr, ContractInfoResponse, Empty, OwnedDeps, SystemError, SystemResult,
    WasmQuery,
};
use cosmwasm_std::{BankMsg, Coin, ContractResult as StdContractResult, Response, SubMsg, Uint128};

pub fn deps_with_creator(
    creator: Addr,
    contract_address: Addr,
) -> OwnedDeps<MockStorage, MockApi, MockQuerier, Empty> {
    let mut deps = mock_dependencies();
    let mut querier = MockQuerier::default();
    querier.update_wasm(move |request| match request {
        WasmQuery::ContractInfo { contract_addr } => {
            if contract_addr == contract_address.as_str() {
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
            amount: Uint128::new(*amount),
        }],
    });

    assert_eq!(res.messages[0], send_msg);
}
