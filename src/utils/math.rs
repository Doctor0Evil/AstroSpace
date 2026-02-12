use crate::simulation::Lattice;

pub fn check_invariants(lattice: &Lattice) {
    let total_power: f64 = lattice.sites.iter().map(|s| s.power).sum();
    let total_church: f64 = lattice.sites.iter().map(|s| s.church).sum();
    assert!(total_power <= lattice.params.lambda * total_church, "POWER invariant violated");

    for site in &lattice.sites {
        assert!(site.bioload <= lattice.params.b_max, "Bioload ceiling violated");
    }
    // Additional invariants can be added as const assertions or runtime checks
}
