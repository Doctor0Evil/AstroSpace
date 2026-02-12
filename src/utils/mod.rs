 pub mod crypto;
pub mod math;

# File: src/utils/crypto.rs
use sha2::{Sha256, Digest};

pub fn compute_sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
