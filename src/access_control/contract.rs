use crate::access_control::AccessControl;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, StdResult};
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
