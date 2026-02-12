use serde::{Deserialize, Serialize};
use rand::Rng;
use crate::ledger::Deed;
use crate::utils::math::check_invariants;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Params {
    pub decay_rate: f64,  // e.g., 0.99
    pub mint_rate: f64,   // e.g., 0.01
    pub growth_rate: f64, // e.g., 0.005
    pub b_max: f64,       // Bioload ceiling, 1.0
    pub lambda: f64,      // POWER <= lambda * CHURCH
}

impl Default for Params {
    fn default() -> Self {
        Self {
            decay_rate: 0.99,
            mint_rate: 0.01,
            growth_rate: 0.005,
            b_max: 1.0,
            lambda: 0.8,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Lattice {
    pub sites: Vec<SiteState>,
    pub params: Params,
    pub deeds: Vec<Deed>,
    pub tick: u64,
}

impl Lattice {
    pub fn new(size: usize, params: &Params) -> Self {
        let mut sites = vec![SiteState::new(); size];
        let mut rng = rand::thread_rng();
        for site in sites.iter_mut() {
            site.occupancy = if rng.gen_bool(0.5) { 1.0 } else { 0.0 };
        }
        Self {
            sites,
            params: params.clone(),
            deeds: Vec::new(),
            tick: 0,
        }
    }

    pub fn step(&mut self) {
        let mut new_sites = self.sites.clone();
        for i in 0..self.sites.len() {
            let left = if i > 0 { Some(self.sites[i-1]) } else { None };
            let right = if i < self.sites.len() - 1 { Some(self.sites[i+1]) } else { None };
            new_sites[i].update(&self.params, (left, right));

            // Log a sample deed (e.g., "Repair" if bioload high)
            if new_sites[i].bioload > 0.7 {
                let deed = Deed::new(self.tick, i as u64, "Repair".to_string());
                self.deeds.push(deed);
            }
        }
        self.sites = new_sites;
        check_invariants(self);  // Panic if violated
        self.tick += 1;
    }
}
