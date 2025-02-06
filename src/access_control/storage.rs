use crate::access_control::error::{
    insufficient_permissions_error, no_role_error, sender_is_not_role_admin_error,
};
use crate::permissions::is_super_admin;
use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Deps, DepsMut, Env, Order, StdResult, Storage};
use cw_storage_plus::Map;

pub const DEFAULT_ADMIN_ROLE: &str = "";

#[cw_serde]
pub struct RoleData {
    admin_role: String,
}

impl Default for RoleData {
    fn default() -> Self {
        RoleData {
            admin_role: DEFAULT_ADMIN_ROLE.to_string(),
        }
    }
}

const ROLE: Map<&str, RoleData> = Map::new("roles");
const HAS_ROLE: Map<(&str, &Addr), ()> = Map::new("has_role");

pub struct AccessControl {}

impl AccessControl {
    fn _set_admin_role(
        storage: &mut dyn Storage,
        role: &str,
        new_admin_role: &str,
    ) -> StdResult<()> {
        if new_admin_role == DEFAULT_ADMIN_ROLE {
            ROLE.remove(storage, role);
        } else {
            ROLE.save(
                storage,
                role,
                &RoleData {
                    admin_role: new_admin_role.to_string(),
                },
            )?;
        }

        Ok(())
    }

    pub fn get_admin_role(storage: &dyn Storage, role: &str) -> StdResult<String> {
        Ok(ROLE.may_load(storage, role)?.unwrap_or_default().admin_role)
    }

    pub fn has_role(storage: &dyn Storage, role: &str, address: &Addr) -> bool {
        HAS_ROLE.has(storage, (role, address))
    }

    pub fn ensure_admin_role(deps: &Deps, sender: &Addr, role: &str) -> StdResult<()> {
        let admin_role = Self::get_admin_role(deps.storage, role)?;

        if Self::has_role(deps.storage, &admin_role, sender) {
            return Ok(());
        }

        Err(sender_is_not_role_admin_error(&role))
    }

    pub fn grant_role(
        deps: &mut DepsMut,
        sender: &Addr,
        role: &str,
        address: &Addr,
    ) -> StdResult<()> {
        Self::ensure_admin_role(&deps.as_ref(), sender, role)?;
        Self::storage_grant_role(deps.storage, role, address)?;
        Ok(())
    }

    pub fn storage_grant_role(
        storage: &mut dyn Storage,
        role: &str,
        address: &Addr,
    ) -> StdResult<()> {
        HAS_ROLE.save(storage, (role, address), &())?;
        Ok(())
    }

    pub fn revoke_role(
        deps: &mut DepsMut,
        sender: &Addr,
        role: &str,
        address: &Addr,
    ) -> StdResult<()> {
        Self::ensure_admin_role(&deps.as_ref(), sender, role)?;
        Self::storage_remove_has_role(deps.storage, role, address)?;
        Ok(())
    }

    pub fn storage_remove_has_role(
        storage: &mut dyn Storage,
        role: &str,
        address: &Addr,
    ) -> StdResult<()> {
        HAS_ROLE.remove(storage, (role, address));
        Ok(())
    }

    pub fn change_admin_role(
        deps: DepsMut,
        sender: &Addr,
        role: &str,
        new_admin_role: &str,
    ) -> StdResult<()> {
        Self::ensure_admin_role(&deps.as_ref(), sender, role)?;
        Self::_set_admin_role(deps.storage, role, new_admin_role)
    }

    pub fn ensure_has_role(deps: &Deps, role: &str, address: &Addr) -> StdResult<()> {
        if !Self::has_role(deps.storage, role, address) {
            return Err(no_role_error(address, role));
        }

        Ok(())
    }

    pub fn ensure_has_role_or_superadmin(
        deps: &Deps,
        env: &Env,
        role: &str,
        address: &Addr,
    ) -> StdResult<()> {
        if Self::has_role(deps.storage, role, address) || is_super_admin(deps, env, address)? {
            Ok(())
        } else {
            Err(no_role_error(address, role))
        }
    }

    pub fn ensure_has_any_role(deps: &Deps, roles: Vec<&str>, address: &Addr) -> StdResult<()> {
        for role in roles {
            if Self::has_role(deps.storage, role, address) {
                return Ok(());
            }
        }

        Err(insufficient_permissions_error())
    }

    pub fn range_all_addresses_with_role<'a>(
        storage: &'a dyn Storage,
        role: &str,
    ) -> StdResult<Box<dyn Iterator<Item = StdResult<Addr>> + 'a>> {
        Ok(Box::new(
            HAS_ROLE
                .prefix(role)
                .range(storage, None, None, Order::Ascending)
                .map(|res| res.map(|(addr, _)| addr)),
        ))
    }

    pub fn range_all_roles<'a>(
        storage: &'a dyn Storage,
    ) -> Box<dyn Iterator<Item = StdResult<String>> + 'a> {
        Box::new(
            ROLE.range(storage, None, None, Order::Ascending)
                .map(|res| res.map(|(addr, _)| addr)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::helpers::deps_with_creator;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    const ROLE_A: &str = "role_a";
    const ROLE_B: &str = "role_b";

    #[test]
    fn get_set_admin_role() {
        let mut deps = mock_dependencies();

        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, &ROLE_A).unwrap(),
            DEFAULT_ADMIN_ROLE
        );

        assert!(AccessControl::_set_admin_role(deps.as_mut().storage, &ROLE_A, &ROLE_B).is_ok());
        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, &ROLE_A).unwrap(),
            ROLE_B
        );
    }

    #[test]
    fn test_grant_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user = Addr::unchecked("user".to_string());

        let str_role_a = ROLE_A;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Make creator the admin
        assert!(AccessControl::storage_grant_role(
            deps.as_mut().storage,
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Admin should be able to grant role
        assert!(
            AccessControl::grant_role(&mut deps.as_mut(), &creator, &str_role_a, &user).is_ok()
        );

        // Ensure the user has the role
        assert!(AccessControl::has_role(
            deps.as_mut().storage,
            &str_role_a,
            &user
        ));
    }

    #[test]
    fn test_revoke_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user = Addr::unchecked("user".to_string());

        let str_role_a = ROLE_A;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());
        assert!(AccessControl::storage_grant_role(
            deps.as_mut().storage,
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Admin should be able to grant role
        assert!(
            AccessControl::grant_role(&mut deps.as_mut(), &creator, &str_role_a, &user).is_ok()
        );

        // Ensure the user has the role
        assert!(AccessControl::has_role(
            deps.as_mut().storage,
            &str_role_a,
            &user
        ));

        // Admin should be able to revoke role
        assert!(
            AccessControl::revoke_role(&mut deps.as_mut(), &creator, &str_role_a, &user).is_ok()
        );

        // Ensure the user no longer has the role
        assert!(!AccessControl::has_role(
            deps.as_mut().storage,
            &str_role_a,
            &user
        ));
    }

    #[test]
    fn test_change_role_admin() {
        let creator = Addr::unchecked("owner".to_string());

        let str_role_a = ROLE_A;
        let str_role_b = ROLE_B;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Ensure the role admin is set correctly
        assert_eq!(
            &AccessControl::get_admin_role(deps.as_mut().storage, &str_role_a).unwrap(),
            DEFAULT_ADMIN_ROLE
        );

        AccessControl::storage_grant_role(deps.as_mut().storage, DEFAULT_ADMIN_ROLE, &creator)
            .unwrap();

        // Change the role admin
        assert!(AccessControl::change_admin_role(
            deps.as_mut(),
            &creator,
            &str_role_a,
            &str_role_b
        )
        .is_ok());

        // Ensure the new role admin is set correctly
        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, &str_role_a).unwrap(),
            str_role_b
        );
    }

    /*

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
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &creator, &role).is_ok());

     // Ensure role admin fails for someone who is not the admin
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &other, &role).is_err());
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
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &role_admin, &role).is_ok());

     // Super-admin is not role admin
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &creator, &role).is_err());

     // Ensure role admin fails for someone who is not the admin
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &other, &role).is_err());
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

     // Super-admin is not role admin
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &creator, &role).is_err());

     // Ensure role admin fails for someone who is not the admin
     assert!(AccessControl::ensure_role_admin(&deps.as_ref(), &other, &role).is_err());
    }
    */
}
