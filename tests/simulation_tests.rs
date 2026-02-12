#[cfg(test)]
mod tests {
    use super::super::simulation::{Lattice, Params, PainMetrics, FearRegime};
    use approx::assert_relative_eq;

    #[test]
    fn test_lattice_step() {
        let params = Params::default();
        let mut lattice = Lattice::new(10, &params);
        let initial_power: f64 = lattice.sites.iter().map(|s| s.power).sum();
        lattice.step();
        let new_power: f64 = lattice.sites.iter().map(|s| s.power).sum();
        assert!(new_power > initial_power);
    }

    #[test]
    fn test_pain_metrics() {
        let params = Params::default();
        let mut lattice = Lattice::new(5, &params);
        // Simulate high load
        for site in lattice.sites.iter_mut() {
            site.bioload = 0.8;
            site.trust = 0.6;
            site.fear = 0.7;
        }
        let pain = PainMetrics::compute_from_lattice(&lattice);
        assert_eq!(pain.regime, FearRegime::Tuned);  // Streak proxy ~5
        assert_relative_eq!(pain.overload_freq, 1.0, epsilon = 0.001);
        assert!(pain.is_teachable());
    }
}
