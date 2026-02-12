use crate::model::*;
use rand::Rng;

/// Clamp helper to keep all token values biophysically reasonable.
fn clamp(v: f64, min: f64, max: f64) -> f64 {
    v.max(min).min(max)
}

/// Update FEAR as a restraint regulator depending on local load and trust.[file:2]
fn update_fear(site: &mut Site) {
    let stress_ratio = if site.bio.capacity > 0.0 {
        site.bio.load / site.bio.capacity
    } else {
        0.0
    };

    let avg_trust = (site.trust.left_trust + site.trust.right_trust) / 2.0;

    // Fear increases with stress, decreases with trust.
    let delta = 0.5 * stress_ratio - 0.3 * avg_trust;
    site.tokens.fear = clamp(site.tokens.fear + delta, 0.0, 5.0);
}

/// POWER minting conditioned on CHURCH and FEAR band.[file:2]
fn mint_power(site: &mut Site, constraints: &GlobalConstraints) {
    if !site.occupied {
        return;
    }

    // Desired fear range for legitimate power.
    if site.tokens.fear < constraints.min_fear || site.tokens.fear > constraints.max_fear {
        return;
    }

    // Power is bounded by church.
    let max_power = constraints.max_power_per_church * site.tokens.church;
    if site.tokens.power < max_power {
        let delta = 0.1 * site.tokens.church;
        site.tokens.power = clamp(site.tokens.power + delta, 0.0, max_power);
    }
}

/// TECH grows when POWER is available and load is safe.
fn update_tech(site: &mut Site) {
    if !site.occupied {
        return;
    }
    if site.bio.is_overloaded() {
        return;
    }
    let delta = 0.05 * site.tokens.power;
    site.tokens.tech = clamp(site.tokens.tech + delta, 0.0, 10.0);
}

/// Local cooperative action: share resources with a neighbor, boosting CHURCH and trust,
/// costing POWER and slightly increasing load (sacrifice). This models good deeds.[file:2]
pub fn local_help(world: &mut World, i: SiteIndex, j: SiteIndex) {
    if i >= world.len() || j >= world.len() {
        return;
    }
    if i == j {
        return;
    }

    let (left_idx, right_idx) = if i < j { (i, j) } else { (j, i) };

    let (left_ptr, right_ptr) = {
        let (left_slice, right_slice) = world.sites.split_at_mut(right_idx);
        (&mut left_slice[left_idx], &mut right_slice[0])
    };

    if !left_ptr.occupied || !right_ptr.occupied {
        return;
    }

    if left_ptr.tokens.power < world.policy.local_help_power_cost {
        return;
    }

    left_ptr.tokens.power -= world.policy.local_help_power_cost;
    left_ptr.tokens.church += world.policy.local_help_church_gain;
    right_ptr.tokens.church += 0.5 * world.policy.local_help_church_gain;

    left_ptr.bio.load += world.policy.local_help_load_gain;
    right_ptr.bio.load += 0.5 * world.policy.local_help_load_gain;

    left_ptr.trust.right_trust = clamp(left_ptr.trust.right_trust + 0.2, -1.0, 1.0);
    right_ptr.trust.left_trust = clamp(right_ptr.trust.left_trust + 0.2, -1.0, 1.0);
}

/// Local conflict over territory: increases POWER for the winner but harms CHURCH,
/// trust, and biophysical load for both sides. This is intentionally costly to reflect
/// your emphasis that fighting is rarely morally justified.[file:2]
pub fn local_conflict(world: &mut World, i: SiteIndex, j: SiteIndex) {
    if i >= world.len() || j >= world.len() || i == j {
        return;
    }

    let (left_idx, right_idx) = if i < j { (i, j) } else { (j, i) };
    let (left_ptr, right_ptr) = {
        let (left_slice, right_slice) = world.sites.split_at_mut(right_idx);
        (&mut left_slice[left_idx], &mut right_slice[0])
    };

    if !left_ptr.occupied || !right_ptr.occupied {
        return;
    }

    let mut rng = rand::thread_rng();
    let left_strength = left_ptr.tokens.power + rng.gen_range(0.0..1.0);
    let right_strength = right_ptr.tokens.power + rng.gen_range(0.0..1.0);

    if left_strength > right_strength {
        left_ptr.tokens.power += world.policy.conflict_power_gain;
    } else {
        right_ptr.tokens.power += world.policy.conflict_power_gain;
    }

    left_ptr.tokens.church =
        clamp(left_ptr.tokens.church - world.policy.conflict_church_loss, 0.0, f64::MAX);
    right_ptr.tokens.church =
        clamp(right_ptr.tokens.church - world.policy.conflict_church_loss, 0.0, f64::MAX);

    left_ptr.trust.right_trust = clamp(
        left_ptr.trust.right_trust - world.policy.conflict_trust_penalty,
        -1.0,
        1.0,
    );
    right_ptr.trust.left_trust = clamp(
        right_ptr.trust.left_trust - world.policy.conflict_trust_penalty,
        -1.0,
        1.0,
    );

    left_ptr.bio.load += world.policy.conflict_load_increase;
    right_ptr.bio.load += world.policy.conflict_load_increase;
}

/// Repair action: spend CHURCH to reduce load (sacrifice / restorative deed).
pub fn repair(world: &mut World, i: SiteIndex) {
    if i >= world.len() {
        return;
    }
    let site = &mut world.sites[i];
    if !site.occupied {
        return;
    }
    if site.tokens.church < world.policy.repair_church_cost {
        return;
    }
    site.tokens.church -= world.policy.repair_church_cost;
    site.bio.load = (site.bio.load - world.policy.repair_load_reduction).max(0.0);
}

/// Colonize adjacent empty territory when CHURCH and FEAR are in a safe band
/// and sufficient POWER is available. Colonization consumes POWER and increases load
/// for both the parent site and the new site, reflecting real biophysical cost.[file:2]
pub fn colonize(world: &mut World, i: SiteIndex) {
    if i >= world.len() {
        return;
    }
    let length = world.len();
    let indices = [i.wrapping_sub(1), i + 1];

    for j in indices {
        if j >= length {
            continue;
        }
        let (parent_idx, child_idx) = (i, j);
        if world.sites[child_idx].occupied {
            continue;
        }

        let parent = &mut world.sites[parent_idx];
        if !parent.occupied {
            continue;
        }

        if parent.tokens.church < world.policy.colonization_church_threshold {
            continue;
        }
        if parent.tokens.fear < world.policy.colonization_fear_min
            || parent.tokens.fear > world.policy.colonization_fear_max
        {
            continue;
        }
        if parent.tokens.power < world.policy.colonization_power_cost {
            continue;
        }

        parent.tokens.power -= world.policy.colonization_power_cost;
        parent.bio.load += world.policy.colonization_load_increase;

        let child_capacity = parent.bio.capacity;
        let mut child = Site::empty(child_idx, child_capacity);
        child.occupied = true;
        child.tokens.church = parent.tokens.church * 0.5;
        child.tokens.fear = parent.tokens.fear;
        child.tokens.power = parent.tokens.power * 0.2;
        child.tokens.tech = parent.tokens.tech * 0.5;
        child.bio.load = parent.bio.load * 0.5;
        child.trust = TrustState::neutral();

        world.sites[child_idx] = child;
    }
}

/// Check Neuromorph-GOD global constraints: if violated, block further tech growth
/// by resetting POWER and forcing repair-like states. This keeps the system within
/// safe biophysical bounds in a transparent, non-fictional way.[file:2]
fn enforce_global_constraints(world: &mut World) {
    let mut total_load = 0.0;
    for site in &world.sites {
        total_load += site.bio.load;
    }

    if total_load > world.constraints.max_total_load {
        for site in &mut world.sites {
            if site.occupied {
                site.tokens.power = 0.0;
                site.tokens.tech = 0.0;
            }
        }
    }
}

/// One full world step: local updates, interactions, colonization, and logging.
pub fn step_world(world: &mut World) {
    world.tick += 1;

    let constraints = world.constraints;

    // 1. Update fear and tech locally, mint power.
    for site in &mut world.sites {
        if site.occupied {
            update_fear(site);
            mint_power(site, &constraints);
            update_tech(site);
        }
    }

    // 2. Local interactions: help or conflict between neighbors.
    // Here we bias toward help to reflect your desired learning signal.
    for i in 0..world.len().saturating_sub(1) {
        let j = i + 1;
        if world.sites[i].occupied && world.sites[j].occupied {
            let avg_trust =
                (world.sites[i].trust.right_trust + world.sites[j].trust.left_trust) / 2.0;
            if avg_trust >= 0.0 {
                local_help(world, i, j);
            } else {
                local_conflict(world, i, j);
            }
        }
    }

    // 3. Colonization attempts from occupied sites.
    let occupied_indices: Vec<SiteIndex> = world
        .sites
        .iter()
        .enumerate()
        .filter(|(_, s)| s.occupied)
        .map(|(i, _)| i)
        .collect();

    for idx in occupied_indices {
        colonize(world, idx);
    }

    // 4. Enforce global constraints.
    enforce_global_constraints(world);

    // 5. Log summary metrics for this tick.
    log_step(world);
}

fn log_step(world: &mut World) {
    let mut occupied = 0usize;
    let mut sum_trust = 0.0;
    let mut sum_fear = 0.0;
    let mut sum_church = 0.0;
    let mut sum_power = 0.0;
    let mut sum_tech = 0.0;

    for site in &world.sites {
        if site.occupied {
            occupied += 1;
            sum_trust += (site.trust.left_trust + site.trust.right_trust) / 2.0;
            sum_fear += site.tokens.fear;
            sum_church += site.tokens.church;
            sum_power += site.tokens.power;
            sum_tech += site.tokens.tech;
        }
    }

    let n = occupied as f64;
    let occupied_fraction = occupied as f64 / world.len() as f64;
    let avg_trust = if n > 0.0 { sum_trust / n } else { 0.0 };
    let avg_fear = if n > 0.0 { sum_fear / n } else { 0.0 };
    let avg_church = if n > 0.0 { sum_church / n } else { 0.0 };
    let avg_power = if n > 0.0 { sum_power / n } else { 0.0 };
    let avg_tech = if n > 0.0 { sum_tech / n } else { 0.0 };

    world.history.push(StepLog {
        tick: world.tick,
        occupied_fraction,
        avg_trust,
        avg_fear,
        avg_church,
        avg_power,
        avg_tech,
    });
}
