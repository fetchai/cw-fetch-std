use cosmwasm_std::{Env, StdError, StdResult, Storage};
use cw_storage_plus::Item;

// api paused
const ERR_CONTRACT_PAUSED: &str = "[FET_ERR_CONTRACT_PAUSED] Contract is paused";

const PAUSED_SINCE_BLOCK: Item<u64> = Item::new("paused_since");

pub fn is_paused_since_block(storage: &dyn Storage) -> StdResult<Option<u64>> {
    PAUSED_SINCE_BLOCK.may_load(storage)
}

pub fn pause_contract(storage: &mut dyn Storage, since_block: u64) -> StdResult<()> {
    PAUSED_SINCE_BLOCK.save(storage, &since_block)
}

pub fn resume_contract(storage: &mut dyn Storage) {
    PAUSED_SINCE_BLOCK.remove(storage)
}

pub fn ensure_not_paused(storage: &dyn Storage, env: &Env) -> StdResult<()> {
    // Return error if contract is paused
    if let Some(paused_since_block) = is_paused_since_block(storage)? {
        if env.block.height >= paused_since_block {
            return Err(contract_paused_error());
        }
    }
    Ok(())
}

pub fn contract_paused_error() -> StdError {
    StdError::generic_err(ERR_CONTRACT_PAUSED)
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

        // State before pausing
        assert!(is_paused_since_block(deps.as_ref().storage)
            .unwrap()
            .is_none());
        assert!(
            ensure_not_paused(deps.as_ref().storage, &mock_env_with_height(pause_height)).is_ok()
        );

        // Pause contract
        assert!(pause_contract(deps.as_mut().storage, pause_height).is_ok());

        // Contract is paused
        assert_eq!(
            is_paused_since_block(deps.as_ref().storage)
                .unwrap()
                .unwrap(),
            pause_height
        );

        assert!(ensure_not_paused(
            deps.as_ref().storage,
            &mock_env_with_height(pause_height - 1)
        )
        .is_ok());
        assert_err(
            &ensure_not_paused(deps.as_ref().storage, &mock_env_with_height(pause_height)),
            &contract_paused_error(),
        );
        assert_err(
            &ensure_not_paused(
                deps.as_ref().storage,
                &mock_env_with_height(pause_height + 1),
            ),
            &contract_paused_error(),
        );

        // Resume contract
        resume_contract(deps.as_mut().storage);

        // Contract is resumed
        assert!(is_paused_since_block(deps.as_ref().storage)
            .unwrap()
            .is_none());

        assert!(ensure_not_paused(
            deps.as_ref().storage,
            &mock_env_with_height(pause_height - 1)
        )
        .is_ok());
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
