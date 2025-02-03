use crate::access_control::AccessControl;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use strum::IntoEnumIterator;

#[cw_serde]
pub struct QueryAllAddressesWithRoleResponse {
    pub addresses: Vec<Addr>,
}

#[cw_serde]
pub struct QueryRolesResponse<T> {
    pub roles: Vec<T>,
}

pub fn query_all_addresses_with_role<T: AsRef<str>>(
    deps: Deps,
    role: T,
) -> StdResult<QueryAllAddressesWithRoleResponse> {
    let addresses_with_role: StdResult<Vec<Addr>> =
        AccessControl::get_all_addresses_with_role(deps.storage, &role).collect();

    Ok(QueryAllAddressesWithRoleResponse {
        addresses: addresses_with_role?,
    })
}

pub fn query_roles<T: IntoEnumIterator + AsRef<str>>(
    deps: Deps,
    addr: Addr,
) -> StdResult<QueryRolesResponse<T>> {
    let roles = T::iter() // Automatically iterate over all roles
        .filter(|role| AccessControl::has_role(deps.storage, role, &addr))
        .collect();

    Ok(QueryRolesResponse { roles })
}

pub fn execute_give_role_by_admin_role<T: AsRef<str>>(
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
        &required_sender_role,
        &info.sender,
    )?;
    AccessControl::storage_set_role(deps.storage, &role, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "give_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role.as_ref())
        .add_attribute("addr", addr.to_string()))
}

pub fn execute_take_role_by_admin_role<T: AsRef<str>>(
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
        &required_sender_role,
        &info.sender,
    )?;
    AccessControl::storage_remove_role(deps.storage, &role, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "take_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role.as_ref())
        .add_attribute("addr", addr.to_string()))
}
