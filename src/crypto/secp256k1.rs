use cosmwasm_std::{StdError, StdResult};

pub type CompressedPubkey = [u8; 33];

fn compress_pubkey(uncompressed_pubkey_bytes: &[u8]) -> CompressedPubkey {
    // The first byte is the prefix, followed by 32 bytes for X and 32 bytes for Y
    let x_bytes = &uncompressed_pubkey_bytes[0..32];
    let y_bytes = &uncompressed_pubkey_bytes[32..64];

    // Determine if Y is even or odd for the prefix
    // Y's last byte's least significant bit determines its evenness or oddness
    let prefix_byte = if (y_bytes[31] & 1) == 0 { 0x02 } else { 0x03 };

    // Create the compressed public key array
    let mut compressed_pubkey = [0u8; 33];
    compressed_pubkey[0] = prefix_byte;
    compressed_pubkey[1..].copy_from_slice(x_bytes);

    compressed_pubkey
}

pub fn to_compressed_key(pubkey: &[u8]) -> StdResult<CompressedPubkey> {
    match pubkey.len() {
        // Compressed pubkey
        33 => pubkey
            .try_into()
            .map_err(|_| pubkey_error(&"Conversion error")),

        // Uncompressed without checksum
        64 => Ok(compress_pubkey(pubkey)),

        // Uncompressed with checksum
        65 => Ok(compress_pubkey(&pubkey[1..])),
        _ => Err(pubkey_error(&"Wrong len")),
    }
}

// Error
pub fn pubkey_error<T: std::fmt::Display>(err: &T) -> StdError {
    StdError::generic_err(format!("Secp256k1 pubkey error {}", err))
}
