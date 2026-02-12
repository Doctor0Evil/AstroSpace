use crate::model::{Site, World};
use serde::{Deserialize, Serialize};

/// Air-Net state per site: persistent, transient, and petrified arial components.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AirNetState {
    /// Persistent background aerial load (normalized 0–1).
    pub persistent: f64,
    /// Transient plume or short-lived airborne load (0–1).
    pub transient: f64,
    /// Petrified / deposited legacy load (0–1).
    pub petrified: f64,
    /// Effective "arial-chemical gravity" controlling diffusion & settling (0–1).
    pub gravity: f64,
}

impl AirNetState {
    pub fn zero() -> Self {
        Self {
            persistent: 0.0,
            transient: 0.0,
            petrified: 0.0,
            gravity: 0.5,
        }
    }
}

/// Parameters controlling Air-Net dynamics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AirNetParams {
    /// Maximum allowed aerial load (persistent + transient) per site.
    pub max_aerial: f64,
    /// Maximum allowed petrified load per site.
    pub max_petrified: f64,
    /// Fraction of transient that diffuses to each neighbor per tick.
    pub diffusion_rate: f64,
    /// Fraction of transient that settles into petrified per tick (scaled by gravity).
    pub settling_rate: f64,
    /// Fraction of transient that is removed by natural decay per tick.
    pub decay_rate: f64,
    /// Fraction of persistent that slowly relaxes toward background.
    pub background_relax: f64,
    /// Background persistent level toward which air tends to relax.
    pub background_level: f64,
}

impl Default for AirNetParams {
    fn default() -> Self {
        Self {
            max_aerial: 1.0,
            max_petrified: 1.0,
            diffusion_rate: 0.05,
            settling_rate: 0.02,
            decay_rate: 0.02,
            background_relax: 0.005,
            background_level: 0.0,
        }
    }
}

/// Extend Site with an Air-Net field by composition-style helper.
pub trait HasAirNet {
    fn air(&self) -> &AirNetState;
    fn air_mut(&mut self) -> &mut AirNetState;
}

impl HasAirNet for Site {
    fn air(&self) -> &AirNetState {
        &self.air
    }
    fn air_mut(&mut self) -> &mut AirNetState {
        &mut self.air
    }
}

/// Extend World with Air-Net parameters.
pub trait HasAirNetParams {
    fn air_params(&self) -> &AirNetParams;
}

impl HasAirNetParams for World {
    fn air_params(&self) -> &AirNetParams {
        &self.air_params
    }
}
