use serde::{Deserialize, Serialize};

use crate::treeoflife::{TreeOfLifeView};          // BLOOD/OXYGEN/.../POWER/FEAR/TECH etc.
use crate::envelope::{BiophysicalEnvelopeSnapshot}; // Raw, normalized axes from BiophysicalEnvelopeSpec
use crate::logging::{EpochLogSink};               // Trait that writes to .evolve.jsonl/.donutloop.aln

/// 0–1 normalized diagnostics for one epoch along the Jetson-Line.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LoadDiagnostics {
    /// External environmental concentration (e.g., PM2.5/chem index), normalized 0–1.
    pub pollution_norm: f32,
    /// Internal metabolic/physiological burden, normalized 0–1.
    pub bioload_index: f32,
    /// Time-integrated pollution × bioload over a fixed horizon, normalized 0–1.
    pub exposure_norm: f32,
}

/// Advisory overload flags (diagnostic-only, no enforcement).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LoadFlags {
    pub overloaded: bool,      // matches NATURE::OVERLOADED semantics
    pub unfair_drain: bool,    // high exposure with low POWER / high hierarchy gap
}

/// Read-only observer for POLLUTION/BIOLOAD/EXPOSURE on the Jetson-Line.
/// Non-actuating: takes snapshots by &ref, returns diagnostics, never mutates capabilities.
pub struct JetsonLineLoadView {
    /// EWMA / window accumulator for exposure integration (diagnostic-only).
    exposure_state: f32,
    /// Configuration thresholds and weights (loaded from ALN shards / config).
    cfg: LoadViewConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadViewConfig {
    /// Hysteresis-aware thresholds, all in 0–1.
    pub pollution_warn: f32,
    pub bioload_warn: f32,
    pub exposure_warn: f32,
    /// EWMA parameter for exposure integration (0–1).
    pub exposure_alpha: f32,
    /// Allometric scaling factor for exposure tolerance vs. TREE capacity proxies.
    pub exposure_capacity_scale: f32,
}

/// Non-actuating trait: can be called by the observer stack, writes only to logs.
pub trait JetsonLineObserver {
    fn observe_epoch(
        &mut self,
        epoch_index: u64,
        envelopes: &BiophysicalEnvelopeSnapshot,
        tree_view: &TreeOfLifeView,
        sink: &mut dyn EpochLogSink,
    );
}

impl JetsonLineLoadView {
    pub fn new(cfg: LoadViewConfig) -> Self {
        Self {
            exposure_state: 0.0,
            cfg,
        }
    }

    /// Pure diagnostic computation: no capability changes, no RoH relaxation.
    fn compute_diagnostics(
        &mut self,
        envelopes: &BiophysicalEnvelopeSnapshot,
        tree_view: &TreeOfLifeView,
    ) -> (LoadDiagnostics, LoadFlags) {
        // 1. POLLUTION: derived axis from envelopes (e.g., env_pm25_norm / env_chem_load_norm)
        let pollution_norm = envelopes
            .axis("env_pm25_norm")
            .or_else(|| envelopes.axis("env_chem_load_norm"))
            .unwrap_or(0.0);

        // 2. BIOLOAD: derived index from existing TREE/envelope axes (no new sensors)
        // Example: weighted combination of thermalload, cognitiveload, inflammation, ecoimpact.
        let thermal = envelopes.axis("thermal_load_norm").unwrap_or(0.0);
        let cognitive = envelopes.axis("cog_load_norm").unwrap_or(0.0);
        let ecoimpact = envelopes.axis("eco_impact_norm").unwrap_or(0.0);

        let bioload_index = (thermal + cognitive + ecoimpact) / 3.0;

        // 3. EXPOSURE: EWMA over pollution * bioload (0–1).
        let instant = pollution_norm * bioload_index;
        self.exposure_state =
            self.cfg.exposure_alpha * instant + (1.0 - self.cfg.exposure_alpha) * self.exposure_state;

        let exposure_norm = self.exposure_state.clamp(0.0, 1.0);

        let diag = LoadDiagnostics {
            pollution_norm: pollution_norm.clamp(0.0, 1.0),
            bioload_index: bioload_index.clamp(0.0, 1.0),
            exposure_norm,
        };

        // 4. Advisory flags based on NATURE predicates (non-actuating).
        let overloaded = diag.bioload_index >= self.cfg.bioload_warn
            || diag.exposure_norm >= self.cfg.exposure_warn;

        // Example unfair_drain: high exposure but low POWER in TREE view.
        let power = tree_view.power; // POWER already normalized 0–1 TREE asset
        let unfair_drain = diag.exposure_norm >= self.cfg.exposure_warn
            && power < self.cfg.exposure_capacity_scale * diag.exposure_norm;

        (diag, LoadFlags { overloaded, unfair_drain })
    }
}

impl JetsonLineObserver for JetsonLineLoadView {
    fn observe_epoch(
        &mut self,
        epoch_index: u64,
        envelopes: &BiophysicalEnvelopeSnapshot,
        tree_view: &TreeOfLifeView,
        sink: &mut dyn EpochLogSink,
    ) {
        let (diag, flags) = self.compute_diagnostics(envelopes, tree_view);

        // Pure logging: write to .evolve.jsonl/.donutloop.aln via EpochLogSink.
        sink.log_load_view(epoch_index, &diag, &flags);
    }
}
