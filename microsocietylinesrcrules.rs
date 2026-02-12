use crate::model::World;
use crate::airnet::update_airnet;

// ...

pub fn step_world(world: &mut World) {
    world.tick += 1;

    let constraints = world.constraints;

    // 1. Existing local token updates (fear, power, tech, bioload, etc.).
    for site in &mut world.sites {
        if site.occupied {
            update_fear(site);
            mint_power(site, constraints);
            update_tech(site);
        }
    }

    // 2. Existing local interactions and colonization.
    //    (local_help, local_conflict, colonize, repair, global constraints, logging, etc.)
    //    ... existing code ...

    // 3. NEW: Air-Net dynamics (aerial, transient, petrified air).
    update_airnet(world);

    // 4. Existing global Neuromorph-GOD enforcement and logging.
    enforce_global_constraints(world);
    log_step(world);
}
