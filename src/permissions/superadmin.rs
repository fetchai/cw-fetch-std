use cosmwasm_std::{Addr, Deps, Env};
use cosmwasm_std::{StdError, StdResult};

// Check if the address is admin of the contract, everyone cannot be admin of the contract
pub fn is_super_admin(deps: &Deps, env: &Env, address: &Addr) -> StdResult<bool> {
    // Check if the address is specified (opposite of the Everyone case)
    if let Some(admin_address) = deps
        .querier
        .query_wasm_contract_info(&env.contract.address)?
        .admin
    {
        return Ok(address == &Addr::unchecked(admin_address));
    }
    Ok(false)
}

pub fn ensure_super_admin(deps: &Deps, env: &Env, address: &Addr) -> StdResult<()> {
    if !is_super_admin(deps, env, address)? {
        return Err(not_super_admin_error());
    }

    Ok(())
}

pub fn not_super_admin_error() -> StdError {
    StdError::generic_err("Sender is not a super-admin.")
}
