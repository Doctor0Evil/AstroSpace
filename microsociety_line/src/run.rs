use crate::model::*;
use crate::rules::step_world;
use serde::{Deserialize, Serialize};

/// Episode is a replayable, auditable container for a single run,
/// aligned with the "Episode" pattern you already use.[file:2]
#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    pub label: String,
    pub world: World,
}

impl Episode {
    pub fn new(label: impl Into<String>, world: World) -> Self {
        Self {
            label: label.into(),
            world,
        }
    }

    pub fn run_for_ticks(&mut self, ticks: Tick) {
        for _ in 0..ticks {
            step_world(&mut self.world);
        }
    }

    pub fn save_json(&self, path: &str) -> std::io::Result<()> {
        let data = serde_json::to_vec_pretty(self).expect("serialize episode");
        std::fs::write(path, data)
    }
}

/// Helper to build a simple demo world:
/// a small Jetson_Line with one seed society in the middle.
pub fn demo_world(length: usize) -> World {
    let constraints = GlobalConstraints {
        max_power_per_church: 2.0,
        max_total_load: 500.0,
        min_fear: 0.2,
        max_fear: 2.0,
    };

    let policy = Policy {
        colonization_church_threshold: 3.0,
        colonization_fear_min: 0.3,
        colonization_fear_max: 1.5,
        colonization_power_cost: 1.0,
        colonization_load_increase: 1.0,
        local_help_church_gain: 0.5,
        local_help_power_cost: 0.2,
        local_help_load_gain: 0.3,
        conflict_power_gain: 0.5,
        conflict_church_loss: 0.5,
        conflict_trust_penalty: 0.4,
        conflict_load_increase: 1.0,
        repair_church_cost: 0.5,
        repair_load_reduction: 1.0,
    };

    let mut world = World::new(length, 10.0, constraints, policy);

    // Seed a central occupied site, representing a micro-society that learns to share.
    let mid = length / 2;
    {
        let site = &mut world.sites[mid];
        site.occupied = true;
        site.tokens.church = 5.0;
        site.tokens.fear = 0.8;
        site.tokens.power = 1.0;
        site.tokens.tech = 0.5;
        site.bio.load = 1.0;
        site.trust.left_trust = 0.0;
        site.trust.right_trust = 0.0;
    }

    world
}
