use cosmwasm_std::Addr;

use crate::crypto::encoding::encode_bech32;
use crate::crypto::hashing::{ripemd160, sha256};

pub fn pubkey_to_address(pubkey: &[u8], prefix: &str) -> Addr {
    let hash = ripemd160(&sha256(pubkey));

    let mut addr = [0u8; 20];
    addr.copy_from_slice(&hash[..]);

    Addr::unchecked(encode_bech32(prefix, &addr).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::encoding::parse_bech32;

    #[test]
    fn test_pubkey_to_address() {
        // Test if encoded pubkey can be decoded and converted to address

        let pubkey_str =
            "pub1qv6wrktsr7hng9rmmjqa2yfqj0cg7w43n0qkq3xuqmgxu6ewnyyjykzgyam".to_string();
        let address = Addr::unchecked("fetch1967p3vkp0yngdfturv4ypq2p4g760ml705wcxy".to_string());

        // Get pubkey in bytes
        let pubkey_bytes = parse_bech32(&pubkey_str, "pub").unwrap();
        // Convert pubkey bytes to address
        let recovered_addr = pubkey_to_address(&pubkey_bytes, "fetch");

        assert_eq!(recovered_addr, address);
    }
}
