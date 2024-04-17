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
