 pub mod crypto;
pub mod math;

# File: src/utils/crypto.rs
use sha2::{Sha256, Digest};

pub fn compute_sha256_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

# File: src/utils/math.rs
use crate::simulation::Lattice;

pub fn check_invariants(lattice: &Lattice) {
    let total_power: f64 = lattice.sites.iter().map(|s| s.power).sum();
    let total_church: f64 = lattice.sites.iter().map(|s| s.church).sum();
    assert!(total_power <= lattice.params.lambda * total_church, "POWER invariant violated");

    for site in &lattice.sites {
        assert!(site.bioload <= lattice.params.b_max, "Bioload ceiling violated");
    }
    // Additional invariants can be added as const assertions or runtime checks
}

# File: tests/simulation_tests.rs
#[cfg(test)]
mod tests {
    use super::super::simulation::{Lattice, Params, PainMetrics, FearRegime};
    use approx::assert_relative_eq;

    #[test]
