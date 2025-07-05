use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PVPConfig {
    /// Whether PVP is enabled.
    pub enabled: bool,
    /// Whether to use the red hurt animation and FOV bobbing.
    pub hurt_animation: bool,
    /// Whether players in creative mode are protected against PVP.
    pub protect_creative: bool,
    /// Whether PVP knockback is enabled.
    pub knockback: bool,
    /// Whether players swing when attacking.
    pub swing: bool,
    /// The type of combat mechanics that are used by default. Options: "Legacy" (MC 1.7.10), "Classic" (MC 1.8), "Modern" (Current)
    pub combat_type: String,
    /// 2.0 by default.
    pub friction: f64,
    /// 0.4 by default.
    pub horizontal_kb: f64,
    /// 0.4 by default.
    pub vertical_kb: f64,
    /// 0.4000000059604645 by default.
    pub vertical_limit: f64,
    /// 0.5 by default.
    pub extra_horizontal_kb: f64,
    /// 0.1 by default.
    pub extra_vertical_kb: f64,
}

impl Default for PVPConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hurt_animation: true,
            protect_creative: true,
            knockback: true,
            swing: true,
            combat_type: String::from("Modern"),
            friction: 2.0,
            horizontal_kb: 0.4,
            vertical_kb: 0.4,
            vertical_limit: 0.4000000059604645,
            extra_horizontal_kb: 0.5,
            extra_vertical_kb: 0.1,
        }
    }
}
