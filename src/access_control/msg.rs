use crate::access_control::{
    execute_grant_role, execute_renounce_role, execute_revoke_role, query_admin_role,
    query_has_role,
};
use crate::access_control::{QueryAdminRoleResponse, QueryHasRoleResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cw_serde]
pub enum AccessControlExecuteMsg {
    GrantRole { role: String, addr: Addr },

    RevokeRole { role: String, addr: Addr },

    RenounceRole { role: String },
}
pub fn handle_access_control_execute_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccessControlExecuteMsg,
) -> StdResult<Response> {
    match msg {
        AccessControlExecuteMsg::GrantRole { role, addr } => {
            execute_grant_role(deps, env, info, role, addr)
        }
        AccessControlExecuteMsg::RevokeRole { role, addr } => {
            execute_revoke_role(deps, env, info, role, addr)
        }
        AccessControlExecuteMsg::RenounceRole { role } => {
            execute_renounce_role(deps, env, info, role)
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum AccessControlQueryMsg {
    #[returns(QueryAdminRoleResponse)]
    QueryAdminRole { role: String },

    #[returns(QueryHasRoleResponse)]
    QueryHasRole { addr: Addr, role: String },
}

pub enum AccessControlQueryResponse {
    QueryAdminRole(QueryAdminRoleResponse),
    QueryHasRole(QueryHasRoleResponse),
}

pub fn handle_access_control_query_msg(
    deps: Deps,
    _env: Env,
    msg: AccessControlQueryMsg,
) -> StdResult<AccessControlQueryResponse> {
    match msg {
        AccessControlQueryMsg::QueryAdminRole { role } => {
            query_admin_role(deps, role).map(AccessControlQueryResponse::QueryAdminRole)
        }
        AccessControlQueryMsg::QueryHasRole { addr, role } => {
            query_has_role(deps, role, addr).map(AccessControlQueryResponse::QueryHasRole)
        }
    }
}
