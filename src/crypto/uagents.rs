pub const AGENT_ADDRESS_PREFIX: &str = "agent";
pub const SIGNATURE_PREFIX: &str = "sig";

pub fn encode_length_prefixed(data: &[u8]) -> Vec<u8> {
    // return u64_be_data_length: [u8] + data: [u8]
    let mut result: Vec<u8> = Vec::new();
    result.append(&mut (data.len() as u64).to_be_bytes().to_vec());
    result.append(&mut data.to_vec());
    result
}
