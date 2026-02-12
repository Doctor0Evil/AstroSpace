mod deed;

pub use deed::Deed;

# File: src/ledger/deed.rs
use serde::{Deserialize, Serialize};
use crate::utils::crypto::compute_sha256_hash;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Deed {
    pub id: String,
    pub tick: u64,
    pub site_id: u64,
    pub deed_type: String,
    pub moral_score: f64,  // [-1,1]
    pub rationale: String,
}

impl Deed {
    pub fn new(tick: u64, site_id: u64, deed_type: String) -> Self {
        let rationale = format!("Deed at tick {} for site {}: {}", tick, site_id, deed_type);
        let moral_score = if deed_type == "Repair" { 0.8 } else { -0.2 };
        let serialized = format!("{}{}{}", tick, site_id, deed_type);
        let id = compute_sha256_hash(serialized.as_bytes());
        Self {
            id,
            tick,
            site_id,
            deed_type,
            moral_score,
            rationale,
        }
    }
}
