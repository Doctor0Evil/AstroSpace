use crate::airnet_state::AirNetState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteState {
    // existing fields: energy, roles, Tree-of-Life view, etc.
    // ...
    pub airnet: AirNetState,  // diagnostic-only Air-Net branch
}
