use cosmwasm_schema::schemars::_serde_json::to_string;
use cosmwasm_std::{from_json, StdError, StdResult};
use serde::de::DeserializeOwned;
use serde::Deserialize;

pub const DEFAULT_ADMIN_ROLE: &str = "";


pub enum Role<T: serde::Serialize + for<'a> Deserialize<'a> + DeserializeOwned> {
    Admin,
    Custom(T),
}
impl<T> Role<T>
where
    T: serde::Serialize + for<'a> Deserialize<'a> + Clone, // Ensure T can be deserialized
{
    pub fn to_string(&self) -> StdResult<String> {
        match self {
            Role::Admin => Ok(DEFAULT_ADMIN_ROLE.to_string()),
            Role::Custom(role) => to_string(&role).map_err(|_| StdError::generic_err("Role serialisation failed")),
        }
    }

    pub fn from_string(val: &str) -> StdResult<Self> {
        if val == DEFAULT_ADMIN_ROLE {
            Ok(Role::Admin)
        } else {
            let deserialized: T =
                from_json("\"".to_owned() + val + "\"").map_err(|_| StdError::generic_err("Unknown role"))?;
            Ok(Role::Custom(deserialized))
        }
    }
}
