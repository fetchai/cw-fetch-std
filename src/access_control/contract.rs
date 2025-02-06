use crate::access_control::AccessControl;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cw_serde]
pub struct QueryHasRoleResponse {
    pub has_role: bool,
}

pub fn query_has_role(deps: Deps, addr: Addr, role: String) -> StdResult<QueryHasRoleResponse> {
    Ok(QueryHasRoleResponse {
        has_role: AccessControl::has_role(deps.storage, &role, &addr),
    })
}

pub fn execute_grant_role_by_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    role: String,
    addr: Addr,
    required_sender_role: String,
) -> StdResult<Response> {
    AccessControl::ensure_has_role_or_superadmin(
        &deps.as_ref(),
        &env,
        &required_sender_role,
        &info.sender,
    )?;
    AccessControl::storage_grant_role(deps.storage, &role, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "grant_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role)
        .add_attribute("addr", addr.to_string()))
}

pub fn execute_revoke_role_by_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    role: &str,
    addr: Addr,
    required_sender_role: &str,
) -> StdResult<Response> {
    AccessControl::ensure_has_role_or_superadmin(
        &deps.as_ref(),
        &env,
        required_sender_role,
        &info.sender,
    )?;

    AccessControl::storage_remove_has_role(deps.storage, role, &addr)?;

    Ok(Response::new()
        .add_attribute("action", "revoke_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role)
        .add_attribute("addr", addr.to_string()))
}

pub fn execute_renounce_role<T: Into<String>>(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    role: String,
) -> StdResult<Response> {
    AccessControl::ensure_has_role(&deps.as_ref(), &role, &info.sender)?;
    AccessControl::storage_remove_has_role(deps.storage, &role, &info.sender)?;

    Ok(Response::new()
        .add_attribute("action", "renounce_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role))
}
