use crate::permissions::is_super_admin;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, StdError, StdResult, Storage};
use cw_storage_plus::Map;
use std::marker::PhantomData;

#[cw_serde]
pub struct RoleData {
    role_admin: Option<Addr>,
}

impl Default for RoleData {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RoleData {
    pub fn new(admin: Option<Addr>) -> Self {
        RoleData { role_admin: admin }
    }
}

const ROLE: Map<&str, RoleData> = Map::new("roles");
const HAS_ROLE: Map<(&str, &Addr), ()> = Map::new("has_role");

pub struct AccessControl<T> {
    phantom: PhantomData<T>,
}

impl<T: AsRef<str>> AccessControl<T> {
    pub fn get_role_admin(storage: &dyn Storage, role: &T) -> StdResult<Option<Addr>> {
        Ok(ROLE
            .may_load(storage, role.as_ref())?
            .and_then(|data| data.role_admin))
    }

    fn _set_role_admin(storage: &mut dyn Storage, role: &T, new_admin: &Addr) -> StdResult<()> {
        let mut role_data = ROLE.may_load(storage, role.as_ref())?.unwrap_or_default();
        role_data.role_admin = Some(new_admin.clone());
        ROLE.save(storage, role.as_ref(), &role_data)
    }

    pub fn has_role(storage: &dyn Storage, role: &T, address: &Addr) -> bool {
        HAS_ROLE.has(storage, (role.as_ref(), address))
    }

    pub fn ensure_role_admin(deps: &Deps, env: &Env, sender: &Addr, role: &T) -> StdResult<()> {
        if !is_super_admin(deps, env, sender)? {
            let role_admin = Self::get_role_admin(deps.storage, role)?.ok_or(
                StdError::generic_err(format!("No admin for role {}", role.as_ref())),
            )?;

            if role_admin != sender {
                return Err(StdError::generic_err(format!(
                    "Sender is not admin of role {}",
                    role.as_ref()
                )));
            }
        }
        Ok(())
    }

    pub fn give_role(
        deps: &mut DepsMut,
        env: &Env,
        sender: &Addr,
        role: &T,
        address: &Addr,
    ) -> StdResult<()> {
        Self::ensure_role_admin(&deps.as_ref(), env, sender, role)?;

        HAS_ROLE.save(deps.storage, (role.as_ref(), address), &())?;
        Ok(())
    }

    pub fn take_role(
        deps: &mut DepsMut,
        env: &Env,
        sender: &Addr,
        role: &T,
        address: &Addr,
    ) -> StdResult<()> {
        Self::ensure_role_admin(&deps.as_ref(), env, sender, role)?;

        HAS_ROLE.remove(deps.storage, (role.as_ref(), address));
        Ok(())
    }

    pub fn create_role(
        storage: &mut dyn Storage,
        role: &T,
        role_admin: Option<&Addr>,
    ) -> StdResult<()> {
        if Self::role_exists(storage, role) {
            return Err(StdError::generic_err(format!(
                "Role {} already exist",
                role.as_ref()
            )));
        }

        ROLE.save(
            storage,
            role.as_ref(),
            &RoleData {
                role_admin: role_admin.cloned(),
            },
        )?;

        Ok(())
    }

    pub fn change_role_admin(
        deps: DepsMut,
        env: &Env,
        sender: &Addr,
        role: &T,
        new_admin: &Addr,
    ) -> StdResult<()> {
        Self::ensure_role_admin(&deps.as_ref(), env, sender, role)?;
        Self::_set_role_admin(deps.storage, role, new_admin)
    }

    pub fn role_exists(storage: &dyn Storage, role: &T) -> bool {
        ROLE.has(storage, role.as_ref())
    }

    pub fn ensure_has_role_if_exists(
        storage: &dyn Storage,
        role: &T,
        address: &Addr,
    ) -> StdResult<()> {
        if Self::role_exists(storage, role) {
            Self::ensure_has_role(storage, role, address)?;
        }

        Ok(())
    }

    pub fn ensure_has_role(storage: &dyn Storage, role: &T, address: &Addr) -> StdResult<()> {
        if !Self::has_role(storage, role, address) {
            return Err(StdError::generic_err(format!(
                "Address {} does not have role {}",
                address,
                role.as_ref()
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access_control::role::tests::TestRole::{RoleA, RoleB};
    use crate::testing::helpers::deps_with_creator;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use std::fmt;

    #[cw_serde]
    enum TestRole {
        RoleA,
        RoleB,
    }

    impl AsRef<str> for TestRole {
        fn as_ref(&self) -> &str {
            match self {
                RoleA => "A",
                RoleB => "B",
            }
        }
    }

    impl fmt::Display for TestRole {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.as_ref())
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

    #[test]
    fn test_create_role() {
        let mut deps = mock_dependencies();
        let creator = Addr::unchecked("owner".to_string());
        let role = TestRole::RoleA;

        // Ensure the role does not exist initially
        assert!(!AccessControl::role_exists(deps.as_mut().storage, &role));

        // Create the role
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_ok());

        // Ensure the role admin is set correctly
        assert_eq!(
            &AccessControl::get_role_admin(deps.as_mut().storage, &role)
                .unwrap()
                .unwrap(),
            &creator
        );

        // Trying to create the same role again should fail
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_err());
    }

    #[test]
    fn test_give_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user = Addr::unchecked("user".to_string());
        let role = TestRole::RoleA;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_ok());

        // Admin should be able to give role
        assert!(AccessControl::give_role(&mut deps.as_mut(), &env, &creator, &role, &user).is_ok());

        // Ensure the user has the role
        assert!(AccessControl::has_role(deps.as_mut().storage, &role, &user));
    }

    #[test]
    fn test_take_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user = Addr::unchecked("user".to_string());
        let role = TestRole::RoleA;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_ok());

        // Admin should be able to give role
        assert!(AccessControl::give_role(&mut deps.as_mut(), &env, &creator, &role, &user).is_ok());

        // Ensure the user has the role
        assert!(AccessControl::has_role(deps.as_mut().storage, &role, &user));

        // Admin should be able to take role
        assert!(AccessControl::take_role(&mut deps.as_mut(), &env, &creator, &role, &user).is_ok());

        // Ensure the user no longer has the role
        assert!(!AccessControl::has_role(
            deps.as_mut().storage,
            &role,
            &user
        ));
    }

    #[test]
    fn test_change_role_admin() {
        let creator = Addr::unchecked("owner".to_string());
        let new_admin = Addr::unchecked("new_admin".to_string());
        let role = TestRole::RoleA;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_ok());

        // Ensure the role admin is set correctly
        assert_eq!(
            &AccessControl::get_role_admin(deps.as_mut().storage, &role)
                .unwrap()
                .unwrap(),
            &creator
        );

        // Change the role admin
        assert!(
            AccessControl::change_role_admin(deps.as_mut(), &env, &creator, &role, &new_admin)
                .is_ok()
        );

        // Ensure the new role admin is set correctly
        assert_eq!(
            &AccessControl::get_role_admin(deps.as_mut().storage, &role)
                .unwrap()
                .unwrap(),
            &new_admin
        );
    }

    #[test]
    fn test_ensure_role_admin() {
        let creator = Addr::unchecked("owner".to_string());
        let other = Addr::unchecked("other".to_string());
        let role = TestRole::RoleA;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_ok());

        // Ensure role admin passes for the correct admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &creator, &role).is_ok());

        // Ensure role admin fails for someone who is not the admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &other, &role).is_err());
    }

    #[test]
    fn test_super_admin() {
        let creator = Addr::unchecked("owner".to_string());
        let role_admin = Addr::unchecked("role_admin".to_string());
        let other = Addr::unchecked("other".to_string());

        let role = TestRole::RoleA;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(
            AccessControl::create_role(deps.as_mut().storage, &role, Some(&role_admin)).is_ok()
        );

        // Ensure role admin passes for the correct admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &role_admin, &role).is_ok());

        // Ensure super-admin is also role admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &creator, &role).is_ok());

        // Ensure role admin fails for someone who is not the admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &other, &role).is_err());
    }

    #[test]
    fn test_no_admin_role() {
        let creator = Addr::unchecked("owner".to_string());
        let other = Addr::unchecked("other".to_string());

        let role = TestRole::RoleA;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(AccessControl::create_role(deps.as_mut().storage, &role, None).is_ok());

        // Ensure super-admin is only role admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &creator, &role).is_ok());

        // Ensure role admin fails for someone who is not the admin
        assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &env, &other, &role).is_err());
    }
}
