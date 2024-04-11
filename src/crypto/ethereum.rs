use cosmwasm_std::{Addr, Api, StdError};
use std::convert::TryInto;

use crate::crypto::hashing::{keccak, KeccakDigest};
use base64::{engine::general_purpose, Engine as _};

pub type EthAddress = [u8; 20];

/// Wraps and input message in the ETH personal message format
///
/// # Arguments
///
/// * `msg` - The message to wrap
///
fn build_eth_msg_for_signing(canonical_msg: &str) -> String {
    format!(
        "\x19Ethereum Signed Message:\n{}{}",
        canonical_msg.len(),
        canonical_msg
    )
}

/// Computes the message and digest that should be signed by the user
pub fn compute_eth_msg_digest(canonical_msg: &str) -> KeccakDigest {
    let wrapped = build_eth_msg_for_signing(canonical_msg);
    keccak(wrapped.as_bytes())
}

/// Checks if the slice is not all zeros
fn is_non_zero(data: &[u8]) -> bool {
    for &v in data {
        if v != 0 {
            return true;
        }
    }

    false
}

/// Parse the input eth address and generate a binary version
///
/// # Arguments
///
/// * `eth_address` - The ETH address to be parsed
///
pub fn parse_eth_address(eth_address: &str) -> Result<EthAddress, StdError> {
    // decode and drop the 0x prefix
    let decoded = if let Some(stripped) = eth_address.strip_prefix("0x") {
        hex::decode(stripped)
    } else {
        hex::decode(eth_address)
    };

    let decoded = decoded.map_err(|err| addresses_error(&err))?;
    let address: EthAddress = decoded
        .try_into()
        .map_err(|_| addresses_error(&"parsing error"))?;

    // do not allow an all zero address
    if !is_non_zero(&address) {
        return Err(addresses_error(&"is zero"));
    }

    Ok(address)
}

/// Parse the input ETH style signature and split into raw signature and recovery code
///
/// # Arguments
///
/// * `signature` - The base64 encoded ETH style signature
///
fn parse_eth_signature(signature: &str) -> Result<(u8, Vec<u8>), StdError> {
    let mut unpacked_signature = general_purpose::STANDARD
        .decode(signature)
        .map_err(|err| signature_error(&err))?;
    if unpacked_signature.len() != 65 {
        return Err(signature_error(&"Wrong length"));
    }

    // extract the recovery code
    let mut recovery_code = unpacked_signature
        .pop()
        .ok_or(signature_error(&"Wrong recovery code"))?;
    if recovery_code >= 27u8 {
        recovery_code -= 27u8;
    }

    // validate the recovery code
    let valid_recovery_code = recovery_code == 0 || recovery_code == 1;
    if !valid_recovery_code {
        return Err(signature_error(&"Unrecoverable signature"));
    }

    Ok((recovery_code, unpacked_signature))
}

pub fn pubkey_to_eth_address(eth_pubkey: &[u8]) -> Result<EthAddress, StdError> {
    keccak(&eth_pubkey[1..])[12..]
        .try_into()
        .map_err(|err| signature_error(&err))
}

/// Checks the provided signature matches the specified native and ETH addresses
///
/// # Arguments
///
/// * `destination_address` - The native address in this linking
/// * `eth_address` - The ETH address in this linking
///
pub fn check_registration(
    api: &dyn Api,
    destination_address: &Addr,
    eth_address: &str,
    signature: &str,
) -> Result<EthAddress, StdError> {
    // compute the expected message and then the digest for it

    let msg = format!("Associate {} with {}", destination_address, eth_address);
    let msg_hash = compute_eth_msg_digest(&msg);

    let recovered_public_key = recover_pubkey(api, &msg_hash, signature)?;
    let recovered_address = pubkey_to_eth_address(&recovered_public_key)?;

    let address = parse_eth_address(eth_address)?;

    // compare the addresses
    if recovered_address != address {
        return Err(signature_error(&"unverifiable signature"));
    }

    Ok(address)
}

pub fn recover_pubkey(
    api: &dyn Api,
    msg_hash: &[u8],
    signature: &str,
) -> Result<Vec<u8>, StdError> {
    // parse the eth style signature, extracting the recovery param from r and s
    let (recovery_param, raw_signature) = parse_eth_signature(signature)?;

    // recover the public key from the signature
    let recovered_public_key = api
        .secp256k1_recover_pubkey(msg_hash, &raw_signature, recovery_param)
        .map_err(|err| signature_error(&err))?;

    if recovered_public_key.len() != 65 {
        return Err(signature_error(&"Wrong length"));
    }
    if recovered_public_key[0] != 4 {
        return Err(signature_error(&"First byte not 0x04"));
    }

    Ok(recovered_public_key)
}

// Errors

pub fn signature_error<T: std::fmt::Display>(err: &T) -> StdError {
    StdError::generic_err(format!("Eth signature error {}", err))
}

pub fn addresses_error<T: std::fmt::Display>(err: &T) -> StdError {
    StdError::generic_err(format!("Eth address error {}", err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::cosmos::cosmos_address_from_pubkey;
    use crate::crypto::hashing::compress_pubkey_secp256k1;
    use cosmwasm_std::testing::mock_dependencies;

    #[test]
    fn it_parses_eth_addresses() {
        assert!(parse_eth_address("0xfdsfsdfsdfs").is_err());
        assert!(parse_eth_address("0x12312399fsdf").is_err());
        assert!(parse_eth_address("0x0000000000000000000000000000000000000000").is_err());
        assert!(parse_eth_address("0xBf8a79E9473c314d344859885E9d7d0906Af8420").is_ok());
        assert!(parse_eth_address("Bf8a79E9473c314d344859885E9d7d0906Af8420").is_ok());
    }

    #[test]
    fn it_wraps_eth_sign_messages() {
        let wrapped = build_eth_msg_for_signing("why hello there");
        assert_eq!(&wrapped, "\x19Ethereum Signed Message:\n15why hello there")
    }

    #[test]
    fn it_computes_digests() {
        let native = Addr::unchecked("native-address");

        let msg = format!("Associate {} with {}", &native, "eth-address");
        let result = compute_eth_msg_digest(&msg);

        assert_eq!(
            hex::encode(result),
            "41b34fcffd51799f83523fdcf409482012be1c99e0ebde4d21016977b27d56ef"
        );
    }

    #[test]
    fn it_doesnt_parse_bad_signatures() {
        assert!(parse_eth_signature("asdfasdfasdf").is_err());
        assert!(parse_eth_signature("AQIDBAUGBwgJEBESExQVFhcYGSAhIiMkJSYnKCkwMTIzNDU2Nzg5QEFCQ0RFRkdISVBRUlNUVVZXWFlgYWJjZB0=").is_err()); // rc 2
        assert!(parse_eth_signature("AQIDBAUGBwgJEBESExQVFhcYGSAhIiMkJSYnKCkwMTIzNDU2Nzg5QEFCQ0RFRkdISVBRUlNUVVZXWFlgYWJjZB4=").is_err());
        // rc 3
    }

    #[test]
    fn it_does_parse_good_signatures() {
        let (rc, sig) = parse_eth_signature("AQIDBAUGBwgJEBESExQVFhcYGSAhIiMkJSYnKCkwMTIzNDU2Nzg5QEFCQ0RFRkdISVBRUlNUVVZXWFlgYWJjZBs=").unwrap();
        assert_eq!(rc, 0u8);
        assert_eq!(hex::encode(&sig), "01020304050607080910111213141516171819202122232425262728293031323334353637383940414243444546474849505152535455565758596061626364");

        let (rc, sig) = parse_eth_signature("AQIDBAUGBwgJEBESExQVFhcYGSAhIiMkJSYnKCkwMTIzNDU2Nzg5QEFCQ0RFRkdISVBRUlNUVVZXWFlgYWJjZBw=").unwrap();
        assert_eq!(rc, 1u8);
        assert_eq!(hex::encode(&sig), "01020304050607080910111213141516171819202122232425262728293031323334353637383940414243444546474849505152535455565758596061626364");
    }

    #[test]
    fn it_does_parse_ledger_style_signatures() {
        let (rc, sig) = parse_eth_signature("AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyAhIiMkJSYnKCkqKywtLi8wMTIzNDU2Nzg5Ojs8PT4/QAA=").unwrap();
        assert_eq!(rc, 0u8);
        assert_eq!(hex::encode(&sig), "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40");

        let (rc, sig) = parse_eth_signature("AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyAhIiMkJSYnKCkqKywtLi8wMTIzNDU2Nzg5Ojs8PT4/QAE=").unwrap();
        assert_eq!(rc, 1u8);
        assert_eq!(hex::encode(&sig), "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f40");
    }

    #[test]
    fn it_can_verify_a_signature() {
        let deps = mock_dependencies();

        let destination_address = Addr::unchecked("native-address");
        let eth_address = "0x561122493eF141Ba1A16301634d0C898B03a0Cf0";
        let signature = "Mh3dWbNS68m9tjwF5a5zXLi7QEhooNKZZqQ5GsXzXHVCboGkxZqxXzqF32CW2heI+LHLN8j3bze2Pae382OaDBw=";

        assert!(check_registration(
            deps.as_ref().api,
            &destination_address,
            eth_address,
            signature
        )
        .is_ok());
    }

    #[test]
    fn it_can_verify_a_ledger_generated_signature() {
        let deps = mock_dependencies();

        let destination_address = Addr::unchecked("fetch1dhmexsh45m6apvmuy2936dtt3juxphvqwlfv8t");
        let eth_address = "0x29f9bb90ac4f3fab98ad65b0c5472a65d0d9ae9e";
        let signature = "pt/jDRAfygsFrllH27ALqN6Uw+HC1I/jIY3Mcw1Y69JEETFfuO4leIIJ9Rn43DBn0qxAh3YEbRiOB4BfZbC03gE=";

        assert!(check_registration(
            deps.as_ref().api,
            &destination_address,
            eth_address,
            signature
        )
        .is_ok());
    }

    #[test]
    fn it_cant_verify_a_bad_signature() {
        let deps = mock_dependencies();

        let destination_address = Addr::unchecked("native-address2");
        let eth_address = "0xbaCf56506032e9f1BF4c9C92925460DE929fa8d8";
        let signature = "AxGAOevyAdZl13QtjAtNnsv5HKB5WzyWVfuCbPoIN/QJ6ul1Q24ZS1WUhdAmyrMe6vhA87gKcbB9T3yy+eeUfRs=";

        assert!(check_registration(
            deps.as_ref().api,
            &destination_address,
            eth_address,
            signature
        )
        .is_err());
    }

    #[test]
    fn test_encode_bech32() {
        let eth_pubkey =  hex::decode("5c084296dfaeaf815a3a7e4e8688ed4140e403f1cd2d2f545c7a3822007763ae0e547eb989d5eecbfc5acd0204531b38e3bcfab232c506db7a9353d68932ca61").unwrap();
        let expected_fetch_address = "fetch1e6lpplutmnxae8u7le9xsr7r9r4y9rukaf4lx8";

        let compressed_pubkey = compress_pubkey_secp256k1(&eth_pubkey).unwrap();

        let fetch_address = cosmos_address_from_pubkey(&compressed_pubkey, "fetch");
        assert_eq!(fetch_address, expected_fetch_address);
    }
}
