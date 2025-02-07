use cosmwasm_schema::cw_serde;

pub const DEFAULT_ADMIN_ROLE: &str = "";

#[cw_serde]
pub struct RoleData {
    pub admin_role: String,
}

impl Default for RoleData {
    fn default() -> Self {
        RoleData {
            admin_role: DEFAULT_ADMIN_ROLE.to_string(),
        }
    }
}
