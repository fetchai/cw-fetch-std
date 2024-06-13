use bech32::{FromBase32, ToBase32};
use cosmwasm_std::StdError;

pub fn parse_bech32_with_prefix(data: &str, expected_prefix: &str) -> Result<Vec<u8>, StdError> {
    let (prefix, parsed_data) = parse_bech32(data)?;

    if prefix != expected_prefix {
        return Err(prefix_error(expected_prefix, &prefix));
    }

    Ok(parsed_data)
}

pub fn parse_bech32(data: &str) -> Result<(String, Vec<u8>), StdError> {
    let (prefix, parsed_data, _variant) = match bech32::decode(data) {
        Ok(parsed_data) => Ok(parsed_data),
        Err(err) => Err(base32_parsing_error(&err)),
    }?;

    match Vec::<u8>::from_base32(&parsed_data) {
        Ok(res) => Ok((prefix, res)),
        Err(err) => Err(base32_parsing_error(&err)),
    }
}

pub fn encode_bech32(prefix: &str, data: &[u8]) -> Result<String, StdError> {
    let encoded = match bech32::encode(prefix, data.to_base32(), bech32::Variant::Bech32) {
        Ok(encoded) => Ok(encoded),
        Err(err) => Err(base32_parsing_error(&err)),
    }?;

    Ok(encoded)
}

// Errors
pub fn prefix_error(expected: &str, actual: &str) -> StdError {
    StdError::generic_err(format!(
        "Wrong prefix. Expected {}, got {}.",
        expected, actual
    ))
}

pub fn base32_parsing_error<T: std::fmt::Display>(err: &T) -> StdError {
    StdError::generic_err(format!("Base32 parsing failed: {}", err))
}
