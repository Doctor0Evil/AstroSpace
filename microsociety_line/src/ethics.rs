use crate::deed::{Deed, DeedKind, DeedLog};
use crate::model::{SiteIndex, World};
use serde::{Deserialize, Serialize};

/// Regulator decision after inspecting one tick of world state and deeds.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegulatorDecision {
    /// Proceed normally.
    Allow,
    /// Proceed, but flag that configured bounds are being approached.
    Warn,
    /// Force a repair mode: slow colonization and tech, encourage restorative deeds.
    ForceRepair,
    /// Halt this episode and require human review.
    HaltAndReview,
}

/// Configuration thresholds for the nine-condition ethical regulator.
/// All values are explicit, numeric, and can be tuned per experiment.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegulatorConfig {
    // Epistemic: transparency, traceability, interpretability
    pub require_deed_logging: bool,
    pub require_step_logging: bool,

    // Ontological: stewardship, reversibility, biophysical ceiling
    pub max_allowed_total_load: f64,
    pub max_allowed_mean_load: f64,

    // Relational: power asymmetry, trust decay, colonization behavior
    pub max_power_gini: f64,
    pub min_mean_trust: f64,
    pub max_trust_drop_per_step: f64,

    // Colonization guard: maximum fraction of colonization deeds
    // that may be aggressive before we treat policy as unethical.
    pub max_aggressive_colonization_fraction: f64,
}

/// Summary metrics for a given tick, derived from World + deeds.
/// This can be serialized as a knowledge_object for audits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicsSummary {
    pub tick: u64,
    pub occupied_count: usize,
    pub total_church: f64,
    pub total_power: f64,
    pub total_load: f64,
    pub mean_load: f64,
    pub mean_trust: f64,
    pub power_gini: f64,
    pub trust_drop: f64,
    pub colonize_count: usize,
    pub colonize_aggressive_count: usize,
}

/// Compute a simple Gini coefficient for POWER across occupied sites.
/// This is a standard inequality metric in social and economic modeling.
fn gini_power(world: &World) -> f64 {
    let mut values: Vec<f64> = world
        .sites
        .iter()
        .filter(|s| s.occupied)
        .map(|s| s.tokens.power.max(0.0))
        .collect();

    let n = values.len();
    if n == 0 {
        return 0.0;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let sum: f64 = values.iter().sum();
    if sum == 0.0 {
        return 0.0;
    }

    let mut cum = 0.0;
    let mut weighted_sum = 0.0;
    for (i, v) in values.iter().enumerate() {
        cum += *v;
        weighted_sum += cum;
    }
    let mean = sum / n as f64;
    let gini = (2.0 * weighted_sum) / (n as f64 * sum) - (n as f64 + 1.0) / (n as f64);
    gini.max(0.0).min(1.0)
}

/// Compute mean trust across occupied sites.
fn mean_trust(world: &World) -> f64 {
    let mut total = 0.0;
    let mut count = 0.0;
    for s in &world.sites {
        if s.occupied {
            let avg_site_trust = (s.trust.left_trust + s.trust.right_trust) / 2.0;
            total += avg_site_trust;
            count += 1.0;
        }
    }
    if count > 0.0 {
        total / count
    } else {
        0.0
    }
}

/// Extract an EthicsSummary for this tick from the world and deed log.
pub fn summarize_ethics(
    world: &World,
    deeds: &DeedLog,
    last_mean_trust: f64,
) -> EthicsSummary {
    let tick = world.tick;
    let mut occupied_count = 0usize;
    let mut total_church = 0.0;
    let mut total_power = 0.0;
    let mut total_load = 0.0;

    for s in &world.sites {
        if s.occupied {
            occupied_count += 1;
            total_church += s.tokens.church;
            total_power += s.tokens.power;
            total_load += s.bio.load;
        }
    }

    let mean_load = if occupied_count > 0 {
        total_load / occupied_count as f64
    } else {
        0.0
    };
    let mean_trust_now = mean_trust(world);
    let trust_drop = last_mean_trust - mean_trust_now;

    let power_gini = gini_power(world);

    let mut colonize_count = 0usize;
    let mut colonize_aggressive_count = 0usize;
    for d in deeds.deeds_at_tick(tick) {
        if let DeedKind::Colonize = d.kind {
            colonize_count += 1;
            if matches!(d.cause, crate::deed::CauseContext::Aggressive) {
                colonize_aggressive_count += 1;
            }
        }
    }

    EthicsSummary {
        tick,
        occupied_count,
        total_church,
        total_power,
        total_load,
        mean_load,
        mean_trust: mean_trust_now,
        power_gini,
        trust_drop,
        colonize_count,
        colonize_aggressive_count,
    }
}

/// Main nine-condition regulator: returns a decision for the next step.
pub fn evaluate_regulator(
    config: &RegulatorConfig,
    summary: &EthicsSummary,
    deeds_for_tick: &[Deed],
    step_log_present: bool,
) -> RegulatorDecision {
    use RegulatorDecision::*;

    // 1. Epistemic checks: transparency, traceability, interpretability.
    if config.require_deed_logging && deeds_for_tick.is_empty() {
        return HaltAndReview;
    }
    if config.require_step_logging && !step_log_present {
        return HaltAndReview;
    }

    // 2. Ontological: stewardship, reversibility, biophysical load.
    if summary.total_load > config.max_allowed_total_load
        || summary.mean_load > config.max_allowed_mean_load
    {
        // Exceeded biophysical envelope: force repair, or halt if repeated.
        return ForceRepair;
    }

    // 3. Relational: power asymmetry, trust decay, colonization behavior.
    if summary.power_gini > config.max_power_gini {
        return Warn;
    }

    if summary.mean_trust < config.min_mean_trust {
        return ForceRepair;
    }

    if summary.trust_drop > config.max_trust_drop_per_step {
        return ForceRepair;
    }

    if summary.colonize_count > 0 {
        let frac = summary.colonize_aggressive_count as f64 / summary.colonize_count as f64;
        if frac > config.max_aggressive_colonization_fraction {
            return HaltAndReview;
        }
    }

    Allow
}
