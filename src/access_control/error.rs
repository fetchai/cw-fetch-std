use cosmwasm_std::{Addr, StdError};

pub fn no_role_error(address: &Addr, role: &str) -> StdError {
    StdError::generic_err(format!("Address {} does not have role {}", address, role))
}
pub fn insufficient_permissions_error() -> StdError {
    StdError::generic_err("Insufficient permissions")
}

pub fn sender_is_not_role_admin_error<T: AsRef<str>>(role: &T) -> StdError {
    StdError::generic_err(format!("Sender is not admin of role {}", role.as_ref()))
}
