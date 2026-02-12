#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeedEvent {
    // existing fields...
    pub pollution_norm: f32,
    pub bioload_index: f32,
    pub exposure_norm: f32,
    // optional: load_flags_overloaded, load_flags_unfair_drain
}
