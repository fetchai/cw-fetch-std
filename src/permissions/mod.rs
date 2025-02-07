mod private;
mod superadmin;

pub use crate::permissions::superadmin::{
    ensure_super_admin, is_super_admin, not_super_admin_error,
};

pub use crate::permissions::private::{ensure_private, not_self_contract_error};
