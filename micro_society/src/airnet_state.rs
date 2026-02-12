use serde::{Serialize, Deserialize};

/// Read-only, diagnostic Air-Net state for one site on the Jetson-Line.
/// All fields are clamped to [0.0, 1.0] and carry DIAGNOSTIC-ONLY semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirNetState {
    // 1. Aerial (persistent background)
    pub background_gases: BackgroundGases,   // normalized background composition
    pub chronic_pollution_stock: f32,        // E_i_persist \in [0,1]

    // 2. Temporarily_arial (transient events)
    pub plume_intensity: f32,                // E_i_transient \in [0,1]
    pub event_half_life: f32,                // normalized decay rate \in [0,1]

    // 3. Petrified_arial (air memory)
    pub surface_deposit: f32,                // E_i_petrified \in [0,1]
    pub air_history_score: f32,              // cumulative harm archive \in [0,1]

    // Semantic tags
    pub pollutant_flag: bool,
    pub carrier_role: bool,
    pub bio_flag: bool,

    // Arial-chemical gravity (effective mixing / settling strength)
    pub arial_chemical_gravity: f32,         // \in [0,1]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundGases {
    pub co2_norm: f32,   // normalized CO2-equivalent
    pub o3_norm: f32,    // normalized O3-equivalent
    pub other_norm: f32, // aggregate of other long-lived species
}

impl AirNetState {
    pub fn clamped(
        mut background_gases: BackgroundGases,
        chronic_pollution_stock: f32,
        plume_intensity: f32,
        event_half_life: f32,
        surface_deposit: f32,
        air_history_score: f32,
        pollutant_flag: bool,
        carrier_role: bool,
        bio_flag: bool,
        arial_chemical_gravity: f32,
    ) -> Self {
        fn c(x: f32) -> f32 {
            if x.is_nan() { 0.0 } else { x.max(0.0).min(1.0) }
        }

        background_gases.co2_norm  = c(background_gases.co2_norm);
        background_gases.o3_norm   = c(background_gases.o3_norm);
        background_gases.other_norm = c(background_gases.other_norm);

        AirNetState {
            background_gases,
            chronic_pollution_stock: c(chronic_pollution_stock),
            plume_intensity:        c(plume_intensity),
            event_half_life:        c(event_half_life),
            surface_deposit:        c(surface_deposit),
            air_history_score:      c(air_history_score),
            pollutant_flag,
            carrier_role,
            bio_flag,
            arial_chemical_gravity: c(arial_chemical_gravity),
        }
    }
}
