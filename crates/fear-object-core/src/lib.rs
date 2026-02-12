use std::time::Duration;
use bioscale_upgrade_store::{
    HostBudget, UpgradeDescriptor, ThermodynamicEnvelope, MlPassSchedule,
    ReversalConditions, EvidenceBundle, UpgradeDecision, BioscaleUpgradeStore,
};
use cyberswarm_neurostack::bci_hostsnapshot::{BciHostSnapshot, BciSafetyThresholds};
use quantified_learning::biokarma::BioKarmaRiskVector;

/// Localized, biophysical fear object tied to concrete observables.
#[derive(Clone, Debug)]
pub struct FearObject {
    pub id: String,
    /// Corridor or tissue domain (e.g., "visual", "motor", "affect").
    pub domain: String,
    /// Biophysical thresholds that define "unsafe" for this object.
    pub thermo: ThermodynamicEnvelope,
    pub ml_schedule: MlPassSchedule,
    pub reversal: ReversalConditions,
    /// Evidence backing (10 hex tags).
    pub evidence: EvidenceBundle,
    /// Maximum tolerated BioKarma risk for this object.
    pub max_risk: f32,
    /// Minimum safe interval between exposures.
    pub min_interval: Duration,
}

/// State of host with respect to a particular fear object.
#[derive(Clone, Debug)]
pub struct FearObjectState {
    pub last_exposure_at: Option<std::time::SystemTime>,
    pub last_decision: Option<UpgradeDecision>,
    pub biokarma: BioKarmaRiskVector,
}

impl FearObject {
    /// Check whether current host telemetry and risk allow exposure.
    pub fn is_exposure_safe(
        &self,
        snapshot: &BciHostSnapshot,
        host_budget: &HostBudget,
        state: &FearObjectState,
    ) -> bool {
        // Derive safety thresholds from the object's own envelopes.
        let thresholds =
            BciSafetyThresholds::from_descriptors(self.thermo.clone(),
                                                  self.ml_schedule.clone(),
                                                  self.reversal.clone());

        if !thresholds.snapshot_safe(snapshot.clone()) {
            return false;
        }

        // Ensure BioKarma risk is below object-specific cap.
        if state.biokarma.total_risk() > self.max_risk {
            return false;
        }

        // Ensure host still has enough budget to tolerate the exposure.
        let remaining = host_budget.remaining_energy_joules;
        let required: f64 = self
            .evidence
            .sequences
            .iter()
            .map(|tag| tag.estimated_joule_cost())
            .sum();
        if required > remaining {
            return false;
        }

        true
    }

    /// Gate an UpgradeDescriptor through this fear object before execution.
    pub fn evaluate_with_fear_gate<S: BioscaleUpgradeStore>(
        &self,
        store: &S,
        host_budget: HostBudget,
        snapshot: BciHostSnapshot,
        state: &FearObjectState,
        desc: UpgradeDescriptor,
        when: std::time::SystemTime,
    ) -> UpgradeDecision {
        if !self.is_exposure_safe(&snapshot, &host_budget, state) {
            return UpgradeDecision::Denied {
                reason: "FearObject: telemetry, risk, or budget outside envelope".into(),
            };
        }
        store.evaluate_upgrade(host_budget, desc, when)
    }
}
