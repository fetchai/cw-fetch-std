use crate::access_control::error::{no_role_error, sender_is_not_role_admin_error};
use crate::access_control::storage::AccessControlStorage;
use crate::events::ResponseHandler;
use crate::permissions::is_super_admin;
use cosmwasm_std::{Addr, Deps, Env, StdResult, Storage};

pub struct AccessControl {}

impl AccessControl {
    pub fn ensure_is_admin(storage: &dyn Storage, role: &str, sender: &Addr) -> StdResult<()> {
        let admin_role = AccessControlStorage::get_admin_role(storage, role)?;

        if AccessControlStorage::has_role(storage, &admin_role, sender) {
            return Ok(());
        }

        Err(sender_is_not_role_admin_error(role))
    }

    pub fn grant_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        sender: &Addr,
        role: &str,
        grant_to_address: &Addr,
    ) -> StdResult<()> {
        Self::ensure_is_admin(storage, role, sender)?;
        AccessControlStorage::grant_role(storage, response_handler, role, grant_to_address)?;
        Ok(())
    }

    pub fn revoke_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        sender: &Addr,
        role: &str,
        address_to_revoke: &Addr,
    ) -> StdResult<()> {
        Self::ensure_is_admin(storage, role, sender)?;
        AccessControlStorage::revoke_role(storage, response_handler, role, address_to_revoke);
        Ok(())
    }

    pub fn renounce_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        sender: &Addr,
        role: &str,
    ) -> StdResult<()> {
        Self::ensure_has_role(storage, role, sender)?;
        AccessControlStorage::revoke_role(storage, response_handler, role, sender);
        Ok(())
    }

    pub fn change_admin_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        sender: &Addr,
        role: &str,
        new_admin_role: &str,
    ) -> StdResult<()> {
        Self::ensure_is_admin(storage, role, sender)?;
        AccessControlStorage::set_admin_role(storage, response_handler, role, new_admin_role)
    }

    pub fn ensure_has_role(storage: &dyn Storage, role: &str, address: &Addr) -> StdResult<()> {
        if !AccessControlStorage::has_role(storage, role, address) {
            return Err(no_role_error(address, Some(role)));
        }

        Ok(())
    }

    pub fn ensure_has_role_or_superadmin(
        deps: &Deps,
        env: &Env,
        role: &str,
        address: &Addr,
    ) -> StdResult<()> {
        if AccessControlStorage::has_role(deps.storage, role, address)
            || is_super_admin(deps, env, address)?
        {
            Ok(())
        } else {
            Err(no_role_error(address, Some(role)))
        }
    }

    pub fn ensure_has_any_role(
        storage: &dyn Storage,
        roles: Vec<&str>,
        address: &Addr,
    ) -> StdResult<()> {
        for role in roles {
            if AccessControlStorage::has_role(storage, role, address) {
                return Ok(());
            }
        }

        Err(no_role_error(address, None))
    }

    pub fn get_admin_role(storage: &dyn Storage, role: &str) -> StdResult<String> {
        AccessControlStorage::get_admin_role(storage, role)
    }

    pub fn has_role(storage: &dyn Storage, role: &str, address: &Addr) -> bool {
        AccessControlStorage::has_role(storage, role, address)
    }

    pub fn _grant_role_unrestricted(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        role: &str,
        grant_to_address: &Addr,
    ) -> StdResult<()> {
        AccessControlStorage::grant_role(storage, response_handler, role, grant_to_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access_control::DEFAULT_ADMIN_ROLE;
    use crate::testing::helpers::deps_with_creator;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    const ROLE_A: &str = "role_a";
    const ROLE_B: &str = "role_b";

    #[test]
    fn get_set_admin_role() {
        let mut deps = mock_dependencies();
        let creator = Addr::unchecked("owner".to_string());

        assert_eq!(
            AccessControl::get_admin_role(deps.as_mut().storage, &ROLE_A).unwrap(),
            DEFAULT_ADMIN_ROLE
        );

        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        assert!(AccessControl::change_admin_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            &ROLE_A,
            &ROLE_B
        )
        .is_ok());

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
        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Make creator the admin
        assert!(AccessControl::grant_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Admin should be able to grant role
        assert!(AccessControl::grant_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            &str_role_a,
            &user
        )
        .is_ok());

        // Ensure the user has the role
        assert!(AccessControl::has_role(
            deps.as_mut().storage,
            &str_role_a,
            &user,
        ));
    }

    #[test]
    fn test_revoke_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user = Addr::unchecked("user".to_string());

        let str_role_a = ROLE_A;

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());
        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Admin should be able to grant role
        assert!(AccessControl::grant_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            &str_role_a,
            &user
        )
        .is_ok());

        // Ensure the user has the role
        assert!(AccessControl::has_role(
            deps.as_mut().storage,
            &str_role_a,
            &user,
        ));

        // Admin should be able to revoke role
        assert!(AccessControl::revoke_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            &str_role_a,
            &user
        )
        .is_ok());

        // Ensure the user no longer has the role
        assert!(!AccessControl::has_role(
            deps.as_mut().storage,
            &str_role_a,
            &user,
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

        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Change the role admin
        assert!(AccessControl::change_admin_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
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

    #[test]
    fn test_ensure_role_admin() {
        let creator = Addr::unchecked("owner".to_string());
        let other = Addr::unchecked("other".to_string());

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        assert!(AccessControl::ensure_is_admin(deps.as_ref().storage, &ROLE_A, &creator).is_err());

        // Give creator admin role
        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        // Ensure role admin passes for the correct admin
        assert!(AccessControl::ensure_is_admin(deps.as_ref().storage, &ROLE_A, &creator).is_ok());

        // Ensure role admin fails for someone who is not the admin
        assert!(AccessControl::ensure_is_admin(deps.as_ref().storage, &ROLE_A, &other).is_err());

        // Test revoke
        assert_eq!(
            AccessControl::revoke_role(
                deps.as_mut().storage,
                &mut ResponseHandler::default(),
                &other,
                &DEFAULT_ADMIN_ROLE,
                &creator
            )
            .unwrap_err(),
            sender_is_not_role_admin_error(&DEFAULT_ADMIN_ROLE)
        );
        assert!(AccessControl::revoke_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            &DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        assert!(AccessControl::ensure_is_admin(deps.as_ref().storage, &ROLE_A, &creator).is_err());
    }

    #[test]
    fn test_renounce_role() {
        let creator = Addr::unchecked("owner".to_string());
        let user1 = Addr::unchecked("user1".to_string());
        let user2 = Addr::unchecked("user2".to_string());

        let env = mock_env();
        let mut deps = deps_with_creator(creator.clone(), env.contract.address.clone());

        // Give creator admin role
        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &creator
        )
        .is_ok());

        assert!(AccessControl::_grant_role_unrestricted(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            DEFAULT_ADMIN_ROLE,
            &user2
        )
        .is_ok());

        assert!(AccessControl::grant_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            &ROLE_A,
            &user1
        )
        .is_ok());

        assert!(AccessControl::change_admin_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            DEFAULT_ADMIN_ROLE,
            &ROLE_A
        )
        .is_ok());

        assert_eq!(
            AccessControl::revoke_role(
                deps.as_mut().storage,
                &mut ResponseHandler::default(),
                &creator,
                DEFAULT_ADMIN_ROLE,
                &creator
            )
            .unwrap_err(),
            sender_is_not_role_admin_error(&DEFAULT_ADMIN_ROLE)
        );

        assert!(AccessControl::renounce_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &creator,
            DEFAULT_ADMIN_ROLE
        )
        .is_ok());

        assert_eq!(
            AccessControl::revoke_role(
                deps.as_mut().storage,
                &mut ResponseHandler::default(),
                &user2,
                DEFAULT_ADMIN_ROLE,
                &user2
            )
            .unwrap_err(),
            sender_is_not_role_admin_error(&DEFAULT_ADMIN_ROLE)
        );

        assert!(AccessControl::revoke_role(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            &user1,
            DEFAULT_ADMIN_ROLE,
            &user2
        )
        .is_ok());
    }
}
