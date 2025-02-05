use crate::access_control::AccessControl;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cw_serde]
pub struct QueryAllAddressesWithRoleResponse {
    pub addresses: Vec<Addr>,
}

#[cw_serde]
pub struct QueryRolesResponse {
    pub roles: Vec<String>,
}

/*
pub fn query_all_addresses_with_role(
    deps: Deps,
    role: String,
) -> StdResult<QueryAllAddressesWithRoleResponse> {
    let addresses_with_role: StdResult<Vec<Addr>> =
        AccessControl::range_all_addresses_with_role(deps.storage, &role).collect();

    Ok(QueryAllAddressesWithRoleResponse {
        addresses: addresses_with_role?,
    })
}

pub fn query_roles(deps: Deps, addr: Addr) -> StdResult<QueryRolesResponse> {
    let res_all_roles: StdResult<Vec<String>> =
        AccessControl::range_all_roles(deps.storage).collect();
    let all_roles = res_all_roles?;

    let mut addr_roles: Vec<String> = Vec::new();

    for role in all_roles {
        if AccessControl::has_role(deps.storage, &role, &addr) {
            addr_roles.push(role.to_string());
        }
    }

    Ok(QueryRolesResponse { roles: addr_roles })
}

pub fn execute_grant_role_by_admin_role<T: Into<String>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    role: T,
    addr: Addr,
    required_sender_role: T,
) -> StdResult<Response> {
    AccessControl::ensure_has_role_or_superadmin(
        &deps.as_ref(),
        &env,
        required_sender_role,
        &info.sender,
    )?;
    let str_role = role.into();
    AccessControl::storage_set_role(deps.storage, &str_role, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "grant_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", str_role)
        .add_attribute("addr", addr.to_string()))
}

pub fn execute_revoke_role_by_admin_role<T: Into<String>>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    role: T,
    addr: Addr,
    required_sender_role: T,
) -> StdResult<Response> {
    AccessControl::ensure_has_role_or_superadmin(
        &deps.as_ref(),
        &env,
        required_sender_role,
        &info.sender,
    )?;
    let str_role = role.into();
    AccessControl::storage_remove_role(deps.storage, &str_role, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "revoke_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", str_role)
        .add_attribute("addr", addr.to_string()))
}

pub fn execute_renounce_role<T: Into<String>>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    role: T,
) -> StdResult<Response> {
    let str_role = role.into();
    AccessControl::ensure_has_role(&deps.as_ref(), &str_role, &info.sender)?;
    AccessControl::storage_remove_role(deps.storage, &str_role, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "renounce_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", str_role))
}
*/