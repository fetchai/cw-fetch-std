use cosmwasm_std::{Addr, StdError};

pub fn no_role_error(address: &Addr, role: Option<&str>) -> StdError {
    let role_str = role.map(|role| format!(" '{}'", role)).unwrap_or_default();
    StdError::msg(format!(
        "Address {} does not have required role{}",
        address, role_str
    ))
}

pub fn sender_is_not_role_admin_error(role: &str) -> StdError {
    StdError::msg(format!("Sender is not admin of the '{}' role", role))
}
