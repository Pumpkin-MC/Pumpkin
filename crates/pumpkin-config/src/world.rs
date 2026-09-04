use serde::{Deserialize, Serialize};

use crate::{chunk::ChunkConfig, lighting::LightingEngineConfig};

/// Configuration for world and level-specific settings.
///
/// Currently, it includes chunk-related options; more settings may be added later.
#[derive(Deserialize, Serialize, Clone)]
pub struct LevelConfig {
    /// Configuration for chunk behaviour and management.
    pub chunk: ChunkConfig,
    /// Configuration for lighting engine propagation mode.
    #[serde(default)]
    pub lighting: LightingEngineConfig,
    /// Number of ticks between autosave checks. If 0, autosave is disabled.
    #[serde(default = "default_autosave_ticks")]
    pub autosave_ticks: u64,
    // TODO: More options
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            chunk: ChunkConfig::default(),
            lighting: LightingEngineConfig::default(),
            autosave_ticks: default_autosave_ticks(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_level_config_enables_autosave() {
        assert_eq!(LevelConfig::default().autosave_ticks, 6000);
    }

    #[test]
    fn generated_config_round_trip_keeps_autosave_enabled() {
        let generated = toml::to_string(&LevelConfig::default()).unwrap();
        let reloaded: LevelConfig = toml::from_str(&generated).unwrap();
        assert_eq!(reloaded.autosave_ticks, 6000);
    }

    #[test]
    fn an_explicit_zero_still_disables_autosave() {
        let disabled = LevelConfig {
            autosave_ticks: 0,
            ..LevelConfig::default()
        };
        let generated = toml::to_string(&disabled).unwrap();
        let reloaded: LevelConfig = toml::from_str(&generated).unwrap();
        assert_eq!(reloaded.autosave_ticks, 0);
    }
}
