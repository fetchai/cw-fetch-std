use crate::access_control::role::RoleData;
use crate::access_control::{
    AccessControlHasRoleRemovedEvent, AccessControlHasRoleUpdatedEvent,
    AccessControlRoleRemovedEvent, AccessControlRoleUpdatedEvent, DEFAULT_ADMIN_ROLE,
};
use crate::events::ResponseHandler;
use cosmwasm_std::{Addr, Order, StdResult, Storage};
use cw_storage_plus::Map;

const ROLE: Map<&str, RoleData> = Map::new("roles");
const HAS_ROLE: Map<(&str, &Addr), ()> = Map::new("has_role");

pub(crate) struct AccessControlStorage {}

impl AccessControlStorage {
    fn remove_role(storage: &mut dyn Storage, response_handler: &mut ResponseHandler, role: &str) {
        response_handler.add_event(AccessControlRoleRemovedEvent { role });

        ROLE.remove(storage, role);
    }

    fn update_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        role: &str,
        admin_role: &str,
    ) -> StdResult<()> {
        response_handler.add_event(AccessControlRoleUpdatedEvent { role, admin_role });

        ROLE.save(
            storage,
            role,
            &RoleData {
                admin_role: admin_role.to_string(),
            },
        )
    }

    pub fn set_admin_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        role: &str,
        new_admin_role: &str,
    ) -> StdResult<()> {
        if new_admin_role == DEFAULT_ADMIN_ROLE {
            Self::remove_role(storage, response_handler, role);
        } else {
            Self::update_role(storage, response_handler, role, new_admin_role)?;
        }

        Ok(())
    }

    pub fn grant_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        role: &str,
        grant_to_address: &Addr,
    ) -> StdResult<()> {
        response_handler.add_event(AccessControlHasRoleUpdatedEvent {
            role,
            addr: grant_to_address.as_str(),
        });

        HAS_ROLE.save(storage, (role, grant_to_address), &())
    }

    pub fn revoke_role(
        storage: &mut dyn Storage,
        response_handler: &mut ResponseHandler,
        role: &str,
        address_to_revoke: &Addr,
    ) {
        response_handler.add_event(AccessControlHasRoleRemovedEvent {
            role,
            addr: address_to_revoke.as_str(),
        });

        HAS_ROLE.remove(storage, (role, address_to_revoke))
    }

    pub fn get_admin_role(storage: &dyn Storage, role: &str) -> StdResult<String> {
        Ok(ROLE.may_load(storage, role)?.unwrap_or_default().admin_role)
    }

    pub fn has_role(storage: &dyn Storage, address: &Addr, role: &str) -> bool {
        HAS_ROLE.has(storage, (role, address))
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn range_all_roles<'a>(
        storage: &'a dyn Storage,
    ) -> Box<dyn Iterator<Item = StdResult<String>> + 'a> {
        Box::new(
            ROLE.range(storage, None, None, Order::Ascending)
                .map(|res| res.map(|(addr, _)| addr)),
        )
    }
}
