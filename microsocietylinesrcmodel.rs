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
/// One full Air-Net update pass over the 1-D Jetson-Line.
pub fn update_airnet(world: &mut World) {
    let params = world.air_params;
    let len = world.sites.len();

    // Collect current transient for diffusion.
    let mut next_transient = vec![0.0_f64; len];

    for i in 0..len {
        let site = &world.sites[i];
        let a = site.air;

        // Base transient at this site.
        let mut local = a.transient;

        // Simple symmetric diffusion, scaled by gravity (higher gravity -> stronger settling, slightly less diffusion).
        let leak = params.diffusion_rate * (1.0 - a.gravity).clamp(0.0, 1.0) * local;
        let stay = (local - 2.0 * leak).max(0.0);

        next_transient[i] += stay;
        if i > 0 {
            next_transient[i - 1] += leak;
        }
        if i + 1 < len {
            next_transient[i + 1] += leak;
        }
    }

    // Apply settling, decay, and persistent/petrified coupling.
    for i in 0..len {
        let site = &mut world.sites[i];
        let mut air = site.air;

        // Updated transient after diffusion.
        let mut tr = next_transient[i].max(0.0);

        // Settling into petrified (scaled by gravity).
        let settle = params.settling_rate * air.gravity.clamp(0.0, 1.0) * tr;
        // Natural decay of transient.
        let decay = params.decay_rate * tr;

        tr = (tr - settle - decay).max(0.0);

        // Update petrified with settled mass, clamped.
        air.petrified = (air.petrified + settle)
            .min(params.max_petrified)
            .max(0.0);

        // Persistent relaxes toward background (e.g., slow atmospheric equalization).
        let delta_persistent =
            params.background_relax * (params.background_level - air.persistent);
        air.persistent = (air.persistent + delta_persistent)
            .max(0.0)
            .min(params.max_aerial);

        // Total aerial load cannot exceed max_aerial.
        let total_aerial = air.persistent + tr;
        if total_aerial > params.max_aerial {
            let scale = params.max_aerial / total_aerial;
            air.persistent *= scale;
            tr *= scale;
        }

        air.transient = tr;

        // Feed Air-Net into biophysical load as an explicit, bounded contribution.
        // Example: 10% of aerial load contributes to bioload, without exceeding capacity.
        let aerial_load = air.persistent + air.transient + air.petrified;
        let extra_load = 0.1 * aerial_load;
        site.bio.load =
            (site.bio.load + extra_load).min(site.bio.capacity).max(0.0);

        // Commit updated air state.
        site.air = air;
    }
}
