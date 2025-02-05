use std::marker::PhantomData;
use crate::access_control::error::{
    insufficient_permissions_error, no_role_error, role_already_exist_error,
    sender_is_not_role_admin_error,
};
use crate::permissions::is_super_admin;
use cosmwasm_schema::cw_serde;
use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, Order, StdResult, Storage, Empty};
use cw_storage_plus::{Key, Map, PrimaryKey};
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
pub enum Role<T: for<'a> PrimaryKey<'a>> {
    Admin,
    Custom(T),
}

impl<T: for<'a> PrimaryKey<'a>> Clone for Role<T> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<'a, T> PrimaryKey<'a> for Role<T>{
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = ();
    type SuperSuffix = ();

    fn key(&self) -> Vec<Key> {
        todo!()
    }
}


#[cw_serde]
pub struct RoleData<T: for<'a> PrimaryKey<'a>> {
    admin_role: Role<T> ,
}

/*
impl<T> RoleData<T> {
    fn default() -> Self {
        RoleData {
            admin_role: Role::Admin,
        }
    }
}
 */

fn get_roles_map<'a, T: for<'b> PrimaryKey<'b>>() -> Map<'a, Role<T>, RoleData<T>>{
    Map::new("roles")
}

fn get_has_role_map<'a, T: for<'b> PrimaryKey<'b>>() -> Map<'a, (Role<T>, Addr), ()>{
    Map::new("has_role")
}

pub struct AccessControl<T> {
    phantom: PhantomData<T>,
}

impl<'a, T: Serialize + DeserializeOwned + for <'b> PrimaryKey<'b>> AccessControl<T> {
    pub fn get_admin_role(
        storage: &dyn Storage,
        role: Role<T>,
    ) -> StdResult<Option<Role<T>>> {
        Ok(get_roles_map::<T>()
            .may_load(storage, role)?
            .map(|data| data.admin_role))
    }

    fn _set_admin_role(
        storage: &mut dyn Storage,
        role: impl Into<String>,
        new_admin_role: Role<T>,
    ) -> StdResult<()> {
        let str_role = role.into();
        let mut role_data = get_roles_map::<T>().may_load(storage, &str_role)?.unwrap_or_default();
        role_data.admin_role = new_admin_role.into();
        get_roles_map::<T>().save(storage, &str_role, &role_data)
    }

    pub fn has_role(storage: &dyn Storage, role: impl Into<String>, address: &Addr) -> bool {
        get_has_role_map::<T>().has(storage, (&role.into(), address))
    }

    pub fn ensure_role_admin(deps: &Deps, sender: &Addr, role: impl Into<String>) -> StdResult<()> {
        let str_role = role.into();
        if let Some(admin_role) = Self::get_admin_role(deps.storage, &str_role)? {
            if Self::has_role(deps.storage, admin_role, sender) {
                return Ok(());
            }
        }

        Err(sender_is_not_role_admin_error(&str_role))
    }

    /*
    pub fn grant_role(
        deps: &mut DepsMut,
        sender: &Addr,
        role: impl Into<String>,
        address: &Addr,
    ) -> StdResult<()> {
        let str_role = role.into();
        Self::ensure_role_admin(&deps.as_ref(), sender, &str_role)?;
        Self::storage_set_role(deps.storage, &str_role, address)?;
        Ok(())
    }

    pub fn storage_set_role(
        storage: &mut dyn Storage,
        role: impl Into<String>,
        address: &Addr,
    ) -> StdResult<()> {
        HAS_ROLE.save(storage, (&role.into(), address), &())?;
        Ok(())
    }

    pub fn revoke_role(
        deps: &mut DepsMut,
        sender: &Addr,
        role: impl Into<String>,
        address: &Addr,
    ) -> StdResult<()> {
        let str_role = role.into();
        Self::ensure_role_admin(&deps.as_ref(), sender, &str_role)?;
        Self::storage_remove_role(deps.storage, &str_role, address)?;
        Ok(())
    }

    pub fn storage_remove_role(
        storage: &mut dyn Storage,
        role: impl Into<String>,
        address: &Addr,
    ) -> StdResult<()> {
        HAS_ROLE.remove(storage, (&role.into(), address));
        Ok(())
    }

    pub fn create_role(
        storage: &mut dyn Storage,
        role: impl Into<String>,
        admin_role: Option<impl Into<String>>,
    ) -> StdResult<()> {
        let str_role = role.into();
        if Self::role_exists(storage, &str_role) {
            return Err(role_already_exist_error(&str_role));
        }

        ROLE.save(
            storage,
            &str_role,
            &RoleData {
                admin_role: admin_role
                    .map(|role| role.into())
                    .unwrap_or_else(|| DEFAULT_ADMIN_ROLE.to_string()),
            },
        )?;

        Ok(())
    }

    pub fn change_role_admin(
        deps: DepsMut,
        sender: &Addr,
        role: impl Into<String>,
        new_admin_role: impl Into<String>,
    ) -> StdResult<()> {
        let str_role = role.into();
        Self::ensure_role_admin(&deps.as_ref(), sender, str_role.as_str())?;
        Self::_set_admin_role(deps.storage, str_role.as_str(), new_admin_role)
    }

    pub fn role_exists(storage: &dyn Storage, role: impl Into<String>) -> bool {
        ROLE.has(storage, &role.into())
    }

    pub fn ensure_has_role(deps: &Deps, role: impl Into<String>, address: &Addr) -> StdResult<()> {
        let str_role = role.into();
        if Self::has_role(deps.storage, &str_role, address) {
            Ok(())
        } else {
            Err(no_role_error(address, &str_role))
        }
    }

    pub fn ensure_has_role_or_superadmin(
        deps: &Deps,
        env: &Env,
        role: impl Into<String>,
        address: &Addr,
    ) -> StdResult<()> {
        let str_role: String = role.into();
        if Self::has_role(deps.storage, &str_role, address) || is_super_admin(deps, env, address)? {
            Ok(())
        } else {
            Err(no_role_error(address, &str_role))
        }
    }

    pub fn ensure_has_roles(
        deps: &Deps,
        roles: Vec<impl Into<String>>,
        address: &Addr,
    ) -> StdResult<()> {
        for role in roles {
            if Self::has_role(deps.storage, role.into(), address) {
                return Ok(());
            }
        }

        Err(insufficient_permissions_error())
    }

    pub fn range_all_addresses_with_role<'a>(
        storage: &'a dyn Storage,
        role: impl Into<String>,
    ) -> Box<dyn Iterator<Item = StdResult<Addr>> + 'a> {
        Box::new(
            HAS_ROLE
                .prefix(&role.into())
                .range(storage, None, None, Order::Ascending)
                .map(|res| res.map(|(addr, _)| addr)),
        )
    }

    pub fn range_all_roles<'a>(
        storage: &'a dyn Storage,
    ) -> Box<dyn Iterator<Item = StdResult<String>> + 'a> {
        Box::new(
            ROLE.range(storage, None, None, Order::Ascending)
                .map(|res| res.map(|(addr, _)| addr)),
        )
    }

     */
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::access_control::role::tests::TestRole::{RoleA, RoleB};
    use crate::testing::helpers::deps_with_creator;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    enum TestRole {
        RoleA,
        RoleB,
    }

    impl Into<String> for TestRole {
        fn into(self) -> String {
            match self {
                RoleA => "A".to_string(),
                RoleB => "B".to_string(),
            }
        }
    }

    #[test]
    fn get_set_admin_role() {
        let mut deps = mock_dependencies();

        AccessControl::create_role(deps.as_mut().storage, RoleA, None::<TestRole>).unwrap();

        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, RoleA)
                .unwrap()
                .unwrap(),
            DEFAULT_ADMIN_ROLE
        );

        assert!(AccessControl::_set_admin_role(deps.as_mut().storage, RoleA, RoleB).is_ok());
        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, RoleA)
                .unwrap()
                .unwrap(),
            Into::<String>::into(RoleB)
        );
    }

    #[test]
    fn test_create_role() {
        let mut deps = mock_dependencies();
        let creator = Addr::unchecked("owner".to_string());

        // Ensure the role does not exist initially
        assert!(!AccessControl::role_exists(deps.as_mut().storage, RoleA));

        // Create the role
        assert!(AccessControl::create_role(deps.as_mut().storage, RoleA, Some(RoleB)).is_ok());

        // Ensure the role admin is set correctly
        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, RoleA)
                .unwrap()
                .unwrap(),
            Into::<String>::into(RoleB)
        );

        // Trying to create the same role again should fail
        assert!(AccessControl::create_role(deps.as_mut().storage, RoleA, Some(&creator)).is_err());
    }

    #[test]
    fn test_grant_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user = Addr::unchecked("user".to_string());

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Create the role and set admin
        assert!(AccessControl::create_role(deps.as_mut().storage, RoleA, None::<TestRole>).is_ok());

        // Make creator the admin
        assert!(AccessControl::storage_set_role(
            deps.as_mut().storage,
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Admin should be able to grant role
        assert!(AccessControl::grant_role(&mut deps.as_mut(), &creator, RoleA, &user).is_ok());

        // Ensure the user has the role
        assert!(AccessControl::has_role(deps.as_mut().storage, RoleA, &user));
    }

    /*

    #[test]
    fn test_revoke_role() {
     let creator = Addr::unchecked("owner".to_string());
     let user = Addr::unchecked("user".to_string());
     let role = TestRole::RoleA;

     let env = mock_env();
     let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

     // Create the role and set admin
     assert!(AccessControl::create_role(deps.as_mut().storage, &role, Some(&creator)).is_ok());

     // Admin should be able to grant role
     assert!(AccessControl::grant_role(&mut deps.as_mut(), &creator, &role, &user).is_ok());

     // Ensure the user has the role
     assert!(AccessControl::has_role(deps.as_mut().storage, &role, &user));

     // Admin should be able to revoke role
     assert!(AccessControl::revoke_role(&mut deps.as_mut(), &creator, &role, &user).is_ok());

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
         AccessControl::change_role_admin(deps.as_mut(), &creator, &role, &new_admin).is_ok()
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
*/