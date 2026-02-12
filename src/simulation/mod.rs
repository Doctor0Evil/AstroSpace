mod site;
mod lattice;
mod pain;

pub use site::SiteState;
pub use lattice::{Lattice, Params};
pub use pain::{PainMetrics, FearRegime};

# File: src/simulation/site.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct SiteState {
    pub church: f64,    // [0,1]
    pub fear: f64,      // [0,1]
    pub power: f64,     // [0,1]
    pub tech: f64,      // [0,1]
    pub bioload: f64,   // [0,1]
    pub trust: f64,     // [0,1]
    pub occupancy: f64, // [0,1]
}

impl SiteState {
    pub fn new() -> Self {
        Self {
            church: 0.5,
            fear: 0.2,
            power: 0.3,
            tech: 0.4,
            bioload: 0.1,
            trust: 0.8,
            occupancy: 0.0,
        }
    }

    pub fn update(&mut self, params: &super::Params, neighbors: (Option<Self>, Option<Self>)) {
        // Simplified update: decay, minting, load adjustment
        self.church *= params.decay_rate;
        self.fear = (self.fear + 0.05 * self.bioload).min(1.0);
        self.power += params.mint_rate * self.church.min(1.0 - self.power);
        self.tech += params.growth_rate * self.power.min(1.0 - self.tech);
        self.bioload = (self.bioload + 0.1 * self.occupancy).min(params.b_max);
        self.trust -= 0.02 * self.fear.max(0.0);
        self.occupancy += 0.05 * (1.0 - self.fear).min(1.0 - self.occupancy);

        // Neighbor influence (simple diffusion)
        if let Some(left) = neighbors.0 {
            self.trust = (self.trust + 0.1 * left.trust) / 1.1;
        }
        if let Some(right) = neighbors.1 {
            self.trust = (self.trust + 0.1 * right.trust) / 1.1;
        }

        self.clamp();
    }

    fn clamp(&mut self) {
        self.church = self.church.clamp(0.0, 1.0);
        self.fear = self.fear.clamp(0.0, 1.0);
        self.power = self.power.clamp(0.0, 1.0);
        self.tech = self.tech.clamp(0.0, 1.0);
        self.bioload = self.bioload.clamp(0.0, 1.0);
        self.trust = self.trust.clamp(0.0, 1.0);
        self.occupancy = self.occupancy.clamp(0.0, 1.0);
    }
}
