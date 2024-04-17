use cosmwasm_std::{StdError, StdResult};

pub type CompressedPubkey = [u8; 33];

fn compress_pubkey(uncompressed_pubkey_bytes: &[u8]) -> Result<CompressedPubkey, StdError> {
    // The first byte is the prefix, followed by 32 bytes for X and 32 bytes for Y
    let x_bytes = &uncompressed_pubkey_bytes[0..32];
    let y_bytes = &uncompressed_pubkey_bytes[32..64];

    // Determine if Y is even or odd for the prefix
    // Y's last byte's least significant bit determines its evenness or oddness
    let prefix_byte = if (y_bytes[31] & 1) == 0 { 0x02 } else { 0x03 };

    // Create the compressed public key
    let mut compressed_pubkey: Vec<u8> = Vec::with_capacity(33);
    compressed_pubkey.push(prefix_byte);
    compressed_pubkey.extend_from_slice(x_bytes);

    Ok(<[u8; 33]>::try_from(compressed_pubkey).unwrap())
}

pub fn to_compressed_key(pubkey: &[u8]) -> StdResult<CompressedPubkey> {
    match pubkey.len() {
        33 => {
            // Compressed pubkey
            Ok(pubkey.try_into().unwrap())
        }
        64 => {
            // Uncompressed without checksum
            compress_pubkey(pubkey)
        }
        65 => {
            // Uncompressed with checksum
            compress_pubkey(&pubkey[1..])
        }
        _ => Err(pubkey_error(&"Wrong len")),
    }
}

// Error
pub fn pubkey_error<T: std::fmt::Display>(err: &T) -> StdError {
    StdError::generic_err(format!("Secp256k1 pubkey error {}", err))
}
