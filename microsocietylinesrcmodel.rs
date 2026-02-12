use serde::{Deserialize, Serialize};
use crate::airnet::{AirNetState, AirNetParams};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub index: SiteIndex,
    pub occupied: bool,
    pub tokens: TokenState,
    pub bio: BiophysicalState,
    pub trust: TrustState,
    // NEW: Air-Net per-site state
    pub air: AirNetState,
}

impl Site {
    pub fn empty(index: SiteIndex, capacity: f64) -> Self {
        Self {
            index,
            occupied: false,
            tokens: TokenState::zero(),
            bio: BiophysicalState::new(capacity),
            trust: TrustState::neutral(),
            air: AirNetState::zero(), // NEW
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub sites: Vec<Site>,
    pub constraints: GlobalConstraints,
    pub policy: Policy,
    // NEW: Air-Net global parameters
    pub air_params: AirNetParams,
    pub tick: Tick,
    pub history: Vec<StepLog>,
}

impl World {
    pub fn new(
        length: usize,
        base_capacity: f64,
        constraints: GlobalConstraints,
        policy: Policy,
    ) -> Self {
        let mut sites = Vec::with_capacity(length);
        for i in 0..length {
            sites.push(Site::empty(i, base_capacity));
        }
        Self {
            sites,
            constraints,
            policy,
            air_params: AirNetParams::default(), // NEW
            tick: 0,
            history: Vec::new(),
        }
    }
}
