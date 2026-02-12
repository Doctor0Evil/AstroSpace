use serde::{Deserialize, Serialize};

pub type Tick = u64;
pub type SiteIndex = usize;

/// Tokens in the 1-D MicroSociety line, constrained and inspectable.
/// Units are abstract but treated as limited biophysical / social capacities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TokenState {
    pub church: f64,
    pub fear: f64,
    pub power: f64,
    pub tech: f64,
}

impl TokenState {
    pub fn zero() -> Self {
        Self {
            church: 0.0,
            fear: 0.0,
            power: 0.0,
            tech: 0.0,
        }
    }
}

/// Biophysical load / stress at a site: too high means collapse or forced repair.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BiophysicalState {
    pub load: f64,      // accumulated stress / damage
    pub capacity: f64,  // maximum sustainable load
}

impl BiophysicalState {
    pub fn new(capacity: f64) -> Self {
        Self { load: 0.0, capacity }
    }

    pub fn is_overloaded(&self) -> bool {
        self.load > self.capacity
    }
}

/// Simple social trust toward immediate neighbors (left/right).
/// This stays local and reflects your emphasis on social trust and respect.[file:1]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TrustState {
    pub left_trust: f64,
    pub right_trust: f64,
}

impl TrustState {
    pub fn neutral() -> Self {
        Self {
            left_trust: 0.0,
            right_trust: 0.0,
        }
    }
}

/// One cell on the Jetson_Line: 1-D position with tokens, biophysics, and trust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub index: SiteIndex,
    pub occupied: bool,
    pub tokens: TokenState,
    pub bio: BiophysicalState,
    pub trust: TrustState,
}

impl Site {
    pub fn empty(index: SiteIndex, capacity: f64) -> Self {
        Self {
            index,
            occupied: false,
            tokens: TokenState::zero(),
            bio: BiophysicalState::new(capacity),
            trust: TrustState::neutral(),
        }
    }
}

/// Global invariants representing Neuromorph-GOD style safety constraints:
/// they are explicit, numeric, and checkable, not metaphysical claims.[file:2]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GlobalConstraints {
    pub max_power_per_church: f64,
    pub max_total_load: f64,
    pub min_fear: f64,
    pub max_fear: f64,
}

/// Parameters controlling colonization and interaction policies.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Policy {
    pub colonization_church_threshold: f64,
    pub colonization_fear_min: f64,
    pub colonization_fear_max: f64,
    pub colonization_power_cost: f64,
    pub colonization_load_increase: f64,
    pub local_help_church_gain: f64,
    pub local_help_power_cost: f64,
    pub local_help_load_gain: f64,
    pub conflict_power_gain: f64,
    pub conflict_church_loss: f64,
    pub conflict_trust_penalty: f64,
    pub conflict_load_increase: f64,
    pub repair_church_cost: f64,
    pub repair_load_reduction: f64,
}

/// One step of logged dynamics for transparency and replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepLog {
    pub tick: Tick,
    pub occupied_fraction: f64,
    pub avg_trust: f64,
    pub avg_fear: f64,
    pub avg_church: f64,
    pub avg_power: f64,
    pub avg_tech: f64,
}

/// Entire 1-D world (Jetson_Line) with constraints and policies.
#[derive(Debug, Serialize, Deserialize)]
pub struct World {
    pub sites: Vec<Site>,
    pub constraints: GlobalConstraints,
    pub policy: Policy,
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
            tick: 0,
            history: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.sites.len()
    }
}
