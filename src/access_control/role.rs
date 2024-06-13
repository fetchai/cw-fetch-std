/*
abstract contract AccessControl is Context, IAccessControl, ERC165 {
struct RoleData {
    mapping(address account => bool) hasRole;
    bytes32 adminRole;
}

mapping(bytes32 role => RoleData) private _roles;

bytes32 public constant DEFAULT_ADMIN_ROLE = 0x00;

 */

use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Map;
use std::marker::PhantomData;

const ROLE_ADMIN: Map<String, Addr> = Map::new("role_admin");

const HAS_ROLE: Map<(&String, &Addr), ()> = Map::new("has_role");

pub struct AccessControl<T> {
    phantom: PhantomData<T>,
}

impl<T: std::fmt::Display> AccessControl<T> {
    pub fn get_role_admin(storage: &dyn Storage, role: &T) -> StdResult<Option<Addr>> {
        ROLE_ADMIN.may_load(storage, role.to_string())
    }

    fn _set_role_admin(storage: &mut dyn Storage, role: &T, new_admin: &Addr) -> StdResult<()> {
        ROLE_ADMIN.save(storage, role.to_string(), new_admin)
    }

    pub fn has_role(storage: &dyn Storage, role: &T, address: &Addr) -> bool {
        HAS_ROLE.has(storage, (&role.to_string(), address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::testing::mock_dependencies;
    use std::fmt;

    #[cw_serde]
    enum TestRole {
        RoleA,
        RoleB,
    }

    impl TestRole {
        pub fn as_str(&self) -> &str {
            match self {
                TestRole::RoleA => "role_a",
                TestRole::RoleB => "role_b",
            }
        }
    }

    impl fmt::Display for TestRole {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }

    #[test]
    fn get_set_role_admin() {
        let mut deps = mock_dependencies();
        let creator = Addr::unchecked("owner".to_string());

        assert!(
            AccessControl::get_role_admin(deps.as_mut().storage, &TestRole::RoleA)
                .unwrap()
                .is_none()
        );
        assert!(
            AccessControl::_set_role_admin(deps.as_mut().storage, &TestRole::RoleA, &creator)
                .is_ok()
        );
        assert_eq!(
            &AccessControl::get_role_admin(deps.as_mut().storage, &TestRole::RoleA)
                .unwrap()
                .unwrap(),
            &creator
        );
    }
}
