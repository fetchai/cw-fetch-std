use crate::access_control::AccessControl;
use crate::events::ResponseHandler;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cw_serde]
pub struct QueryHasRoleResponse {
    pub has_role: bool,
}

#[cw_serde]
pub struct QueryAdminRoleResponse {
    pub admin_role: String,
}

pub fn query_has_role(deps: Deps, addr: Addr, role: String) -> StdResult<QueryHasRoleResponse> {
    Ok(QueryHasRoleResponse {
        has_role: AccessControl::has_role(deps.storage, &role, &addr),
    })
}

pub fn query_admin_role(deps: Deps, role: String) -> StdResult<QueryAdminRoleResponse> {
    Ok(QueryAdminRoleResponse {
        admin_role: AccessControl::get_admin_role(deps.storage, &role)?,
    })
}

pub fn execute_grant_role_by_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    role: String,
    addr: Addr,
    required_sender_role: &str,
) -> StdResult<Response> {
    AccessControl::ensure_has_role_or_superadmin(
        &deps.as_ref(),
        &env,
        required_sender_role,
        &info.sender,
    )?;

    let mut response_handler = ResponseHandler::default();

    AccessControl::storage_grant_role(deps.storage, &mut response_handler, &role, &addr)?;

    Ok(response_handler
        .into_response()
        .add_attribute("action", "grant_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role)
        .add_attribute("addr", addr.to_string()))
}

pub fn execute_revoke_role_by_admin_role(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    role: String,
    addr: Addr,
    required_sender_role: &str,
) -> StdResult<Response> {
    AccessControl::ensure_has_role_or_superadmin(
        &deps.as_ref(),
        &env,
        required_sender_role,
        &info.sender,
    )?;

    let mut response_handler = ResponseHandler::default();

    AccessControl::storage_revoke_role(deps.storage, &mut response_handler, &role, &addr);

    Ok(response_handler
        .into_response()
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

    let mut response_handler = ResponseHandler::default();

    AccessControl::storage_revoke_role(deps.storage, &mut response_handler, &role, &info.sender);

    Ok(response_handler
        .into_response()
        .add_attribute("action", "renounce_role")
        .add_attribute("sender", info.sender)
        .add_attribute("role", role))
}
