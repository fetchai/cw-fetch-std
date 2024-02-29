use bech32::{self, FromBase32, ToBase32};
use cosmwasm_std::{Addr, StdError};

use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

pub const AGENT_ADDRESS_PREFIX: &str = "agent";
pub const SIGNATURE_PREFIX: &str = "sig";

pub fn encode_length_prefixed(data: &[u8]) -> Vec<u8> {
    // return u64_be_data_length: [u8] + data: [u8]
    let mut result: Vec<u8> = Vec::new();
    result.append(&mut (data.len() as u64).to_be_bytes().to_vec());
    result.append(&mut data.to_vec());
    result
}

pub fn parse_bech32(data: &str, expected_prefix: &str) -> Result<Vec<u8>, StdError> {
    let (prefix, parsed_data, _variant) = match bech32::decode(data) {
        Ok(parsed_data) => Ok(parsed_data),
        Err(err) => Err(base32_parsing_error(&err)),
    }?;

    if prefix != expected_prefix {
        return Err(prefix_error(expected_prefix, &prefix));
    }

    match Vec::<u8>::from_base32(&parsed_data) {
        Ok(res) => Ok(res),
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

pub fn pubkey_to_address(pubkey: &[u8]) -> Addr {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    let hash = hasher.finalize();
    let mut ripemd160 = Ripemd160::new();
    ripemd160.update(hash);
    let hash = ripemd160.finalize();
    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[..]);

    Addr::unchecked(encode_bech32("fetch", &addr).unwrap())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pubkey_to_address() {
        // Test if encoded pubkey can be decoded and converted to address

        let pubkey_str =
            "pub1qv6wrktsr7hng9rmmjqa2yfqj0cg7w43n0qkq3xuqmgxu6ewnyyjykzgyam".to_string();
        let address = Addr::unchecked("fetch1967p3vkp0yngdfturv4ypq2p4g760ml705wcxy".to_string());

        // Get pubkey in bytes
        let pubkey_bytes = parse_bech32(&pubkey_str, "pub").unwrap();
        // Convert pubkey bytes to address
        let recovered_addr = pubkey_to_address(&pubkey_bytes);

        assert_eq!(recovered_addr, address);
    }
}
