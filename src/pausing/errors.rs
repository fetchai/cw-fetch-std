use cosmwasm_std::StdError;

// api paused
const ERR_CONTRACT_PAUSED: &str = "[FET_ERR_CONTRACT_PAUSED] Contract is paused";

pub fn contract_paused_error() -> StdError {
    StdError::msg(ERR_CONTRACT_PAUSED)
}
