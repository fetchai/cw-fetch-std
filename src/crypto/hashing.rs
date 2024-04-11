use cosmwasm_std::StdError;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};
use tiny_keccak::Hasher;

pub type KeccakDigest = [u8; 32];

/// Computes the Eth style Keccak digest for the input data
///
/// # Arguments
///
/// * `data` - The slice to be hashed
///
pub fn keccak(data: &[u8]) -> KeccakDigest {
    let mut output = [0u8; 32];

    let mut hasher = tiny_keccak::Keccak::v256();
    hasher.update(data);
    hasher.finalize(&mut output);

    output
}

pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn ripemd160(data: &[u8]) -> Vec<u8> {
    let mut ripemd160 = Ripemd160::new();
    ripemd160.update(data);
    ripemd160.finalize().to_vec()
}

pub fn compress_pubkey_secp256k1(uncompressed_pubkey_bytes: &[u8]) -> Result<Vec<u8>, StdError> {
    if uncompressed_pubkey_bytes.len() != 64 {
        return Err(pubkey_error(&"Wrong len"));
    }

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

    Ok(compressed_pubkey)
}

// Error
pub fn pubkey_error<T: std::fmt::Display>(err: &T) -> StdError {
    StdError::generic_err(format!("Eth pubkey error {}", err))
}
