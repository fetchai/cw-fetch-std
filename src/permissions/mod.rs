mod superadmin;

pub use crate::permissions::superadmin::{
    ensure_super_admin, is_super_admin, not_super_admin_error,
};
