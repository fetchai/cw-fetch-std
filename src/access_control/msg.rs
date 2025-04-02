use crate::access_control::{execute_grant_role, execute_renounce_role, execute_revoke_role};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Response, StdResult};

#[cw_serde]
pub enum AccessControlMsg {
    GrantRole { role: String, addr: Addr },

    RevokeRole { role: String, addr: Addr },

    RenounceRole { role: String },
}

pub fn handle_access_control_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: AccessControlMsg,
) -> StdResult<Response> {
    match msg {
        AccessControlMsg::GrantRole { role, addr } => {
            execute_grant_role(deps, env, info, role, addr)
        }
        AccessControlMsg::RevokeRole { role, addr } => {
            execute_revoke_role(deps, env, info, role, addr)
        }
        AccessControlMsg::RenounceRole { role } => execute_renounce_role(deps, env, info, role),
    }
}
