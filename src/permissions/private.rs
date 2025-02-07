use cosmwasm_std::{Addr, Env};
use cosmwasm_std::{StdError, StdResult};

const ERR_NOT_SELF_CONTRACT: &str = "[FET_ERR_NOT_SELF] Sender is not a self contract.";

pub fn ensure_private(env: &Env, address: &Addr) -> StdResult<()> {
    if env.contract.address != address {
        return Err(not_self_contract_error());
    }

    Ok(())
}

pub fn not_self_contract_error() -> StdError {
    StdError::generic_err(ERR_NOT_SELF_CONTRACT)
}
