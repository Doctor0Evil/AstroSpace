use crate::model::{SiteIndex, Tick, World};
use serde::{Deserialize, Serialize};

/// What kind of deed occurred. This stays close to your existing actions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DeedKind {
    LocalHelp,
    LocalConflict,
    Colonize,
    Repair,
}

/// Simple cause / justification flags for ethical analysis.
/// You can extend this as you integrate neurolaw-style notions of necessity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CauseContext {
    Defensive,
    Preventive,
    Aggressive,
    Restorative,
    Unknown,
}

/// One logged deed on the Jetson_Line.
/// This is a non-fictional record: pre/post token and load changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deed {
    pub id: u64,
    pub tick: Tick,
    pub kind: DeedKind,
    pub primary_site: SiteIndex,
    pub other_site: Option<SITEIndex>,

    // Biophysical and social deltas (post - pre) for convenience.
    pub delta_church_primary: f64,
    pub delta_church_other: f64,
    pub delta_power_primary: f64,
    pub delta_power_other: f64,
    pub delta_load_primary: f64,
    pub delta_load_other: f64,

    pub delta_trust_primary_to_other: f64,
    pub delta_trust_other_to_primary: f64,

    pub cause: CauseContext,
}

/// A log for one episode: append-only list of deeds.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeedLog {
    next_id: u64,
    pub deeds: Vec<Deed>,
}

impl DeedLog {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            deeds: Vec::new(),
        }
    }

    pub fn record(&mut self, mut deed: Deed) {
        deed.id = self.next_id;
        self.next_id += 1;
        self.deeds.push(deed);
    }

    /// Optional helper: get deeds during a specific tick.
    pub fn deeds_at_tick(&self, tick: Tick) -> impl Iterator<Item = &Deed> {
        self.deeds.iter().filter(move |d| d.tick == tick)
    }
}

/// Utility to compute (approximate) deltas for a local two-site interaction.
/// Call this *before* and *after* applying an action.
pub fn diff_two_sites_deed(
    tick: Tick,
    kind: DeedKind,
    primary_idx: SiteIndex,
    other_idx: Option<SITEIndex>,
    world_before: &World,
    world_after: &World,
    cause: CauseContext,
) -> Deed {
    let p = primary_idx;
    let q = other_idx;

    let (primary_before, primary_after) = (&world_before.sites[p], &world_after.sites[p]);

    let (delta_church_other,
         delta_power_other,
         delta_load_other,
         delta_trust_primary_to_other,
         delta_trust_other_to_primary) = if let Some(j) = q {
        let other_before = &world_before.sites[j];
        let other_after = &world_after.sites[j];

        let d_church = other_after.tokens.church - other_before.tokens.church;
        let d_power = other_after.tokens.power - other_before.tokens.power;
        let d_load = other_after.bio.load - other_before.bio.load;
        let d_trust_p_to_o =
            other_after.trust.left_trust + other_after.trust.right_trust
            - other_before.trust.left_trust - other_before.trust.right_trust;

        // For simplicity, approximate symmetric trust delta.
        let d_trust_o_to_p = d_trust_p_to_o;

        (d_church, d_power, d_load, d_trust_p_to_o, d_trust_o_to_p)
    } else {
        (0.0, 0.0, 0.0, 0.0, 0.0)
    };

    Deed {
        id: 0, // filled by DeedLog::record
        tick,
        kind,
        primary_site: primary_idx,
        other_site: other_idx,
        delta_church_primary: primary_after.tokens.church - primary_before.tokens.church,
        delta_church_other,
        delta_power_primary: primary_after.tokens.power - primary_before.tokens.power,
        delta_power_other,
        delta_load_primary: primary_after.bio.load - primary_before.bio.load,
        delta_load_other,
        delta_trust_primary_to_other,
        delta_trust_other_to_primary,
        cause,
    }
}
