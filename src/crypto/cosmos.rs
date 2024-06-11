use cosmwasm_std::Addr;

use crate::crypto::encoding::encode_bech32;
use crate::crypto::hashing::{ripemd160, sha256};
use crate::crypto::secp256k1::to_compressed_key;
use cosmwasm_std::StdResult;

pub fn cosmos_raw_address_from_pubkey_secp256k1(pubkey: &[u8]) -> StdResult<Vec<u8>> {
    let compressed_pubkey = to_compressed_key(pubkey)?;

    let hash = ripemd160(&sha256(&compressed_pubkey));

    Ok(hash)
}

pub fn cosmos_address(raw_address: &[u8], prefix: &str) -> Addr {
    Addr::unchecked(encode_bech32(prefix, raw_address).unwrap())
}

pub fn cosmos_address_from_pubkey_secp256k1(pubkey: &[u8], prefix: &str) -> StdResult<Addr> {
    Ok(cosmos_address(
        &cosmos_raw_address_from_pubkey_secp256k1(pubkey)?,
        prefix,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encoding::{parse_bech32, parse_bech32_with_prefix};

    #[test]
    fn test_pubkey_to_address() {
        // Test if encoded pubkey can be decoded and converted to address

        let pubkey_str =
            "pub1qv6wrktsr7hng9rmmjqa2yfqj0cg7w43n0qkq3xuqmgxu6ewnyyjykzgyam".to_string();
        let address = Addr::unchecked("fetch1967p3vkp0yngdfturv4ypq2p4g760ml705wcxy".to_string());

        // Get pubkey in bytes
        let pubkey_bytes = parse_bech32_with_prefix(&pubkey_str, "pub").unwrap();
        // Convert pubkey bytes to address
        let recovered_addr = cosmos_address_from_pubkey_secp256k1(&pubkey_bytes, "fetch").unwrap();

        assert_eq!(recovered_addr, address);
    }

    #[test]
    fn test_canonical() {
        let addr_asi = "asi1rhrlzsx9z865dqen8t4v47r99dw6y4vaw76rd9";
        let addr_fet = "fetch1rhrlzsx9z865dqen8t4v47r99dw6y4va4uph0x";

        let (asi_prefix, raw_addr) = parse_bech32(addr_asi).unwrap();
        let recovered_addr = encode_bech32("fetch", &raw_addr).unwrap();

        assert_eq!(asi_prefix, "asi");
        assert_eq!(addr_fet, recovered_addr);

        let contract_addr_fet = "fetch1mxz8kn3l5ksaftx8a9pj9a6prpzk2uhxnqdkwuqvuh37tw80xu6qges77l";
        let contract_addr_asi = "asi1mxz8kn3l5ksaftx8a9pj9a6prpzk2uhxnqdkwuqvuh37tw80xu6qepjszq";

        let (asi_prefix, raw_addr) = parse_bech32(contract_addr_asi).unwrap();
        let recovered_addr = encode_bech32("fetch", &raw_addr).unwrap();

        assert_eq!(asi_prefix, "asi");
        assert_eq!(contract_addr_fet, recovered_addr);
    }
}
