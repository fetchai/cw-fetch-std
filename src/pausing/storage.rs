use crate::events::ResponseHandler;
use crate::pausing::contract_paused_error;
use crate::pausing::events::{ContractPausedEvent, ContractResumedEvent};
use cosmwasm_std::{Env, StdResult, Storage};
use cw_storage_plus::Item;

const PAUSED_SINCE_BLOCK: Item<u64> = Item::new("paused_since");

pub fn paused_since_block(storage: &dyn Storage) -> StdResult<Option<u64>> {
    PAUSED_SINCE_BLOCK.may_load(storage)
}

pub fn is_paused(storage: &dyn Storage, env: &Env) -> StdResult<bool> {
    // Return error if contract is paused
    if let Some(paused_since_block) = paused_since_block(storage)? {
        if env.block.height >= paused_since_block {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn pause_contract(
    storage: &mut dyn Storage,
    response_handler: &mut ResponseHandler,
    since_block: u64,
) -> StdResult<()> {
    response_handler.add_event(ContractPausedEvent {
        since_block: &since_block,
    });
    PAUSED_SINCE_BLOCK.save(storage, &since_block)
}

pub fn resume_contract(storage: &mut dyn Storage, response_handler: &mut ResponseHandler) {
    response_handler.add_event(ContractResumedEvent {});
    PAUSED_SINCE_BLOCK.remove(storage)
}

pub fn ensure_not_paused(storage: &dyn Storage, env: &Env) -> StdResult<()> {
    // Return error if contract is paused
    if is_paused(storage, env)? {
        return Err(contract_paused_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::helpers::{assert_err, mock_env_with_height};
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn test_pausing() {
        let mut deps = mock_dependencies();
        let pause_height = 123u64;
        let env_before_env = mock_env_with_height(pause_height - 1);
        let env_at_paused = mock_env_with_height(pause_height);
        let env_after_paused = mock_env_with_height(pause_height + 1);

        // State before pausing
        assert!(paused_since_block(deps.as_ref().storage).unwrap().is_none());
        assert!(!is_paused(deps.as_ref().storage, &env_at_paused).unwrap());
        assert!(ensure_not_paused(deps.as_ref().storage, &env_at_paused).is_ok());

        // Pause contract
        assert!(pause_contract(
            deps.as_mut().storage,
            &mut ResponseHandler::default(),
            pause_height
        )
        .is_ok());

        // Contract is paused
        assert_eq!(
            paused_since_block(deps.as_ref().storage).unwrap().unwrap(),
            pause_height
        );

        assert!(ensure_not_paused(deps.as_ref().storage, &env_before_env).is_ok());
        assert!(!is_paused(deps.as_ref().storage, &env_before_env).unwrap());

        assert_err(
            &ensure_not_paused(deps.as_ref().storage, &env_at_paused),
            &contract_paused_error(),
        );
        assert!(is_paused(deps.as_ref().storage, &env_at_paused).unwrap());

        assert_err(
            &ensure_not_paused(deps.as_ref().storage, &env_after_paused),
            &contract_paused_error(),
        );

        // Resume contract
        resume_contract(deps.as_mut().storage, &mut ResponseHandler::default());

        // Contract is resumed
        assert!(paused_since_block(deps.as_ref().storage).unwrap().is_none());

        assert!(ensure_not_paused(deps.as_ref().storage, &env_before_env).is_ok());
        assert!(
            ensure_not_paused(deps.as_ref().storage, &mock_env_with_height(pause_height)).is_ok()
        );
        assert!(ensure_not_paused(
            deps.as_ref().storage,
            &mock_env_with_height(pause_height + 1)
        )
        .is_ok());
    }
}
