use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

use crate::{chunk::ChunkConfig, lighting::LightingEngineConfig};

/// Controls the CPU workers used for background chunk generation.
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct GenerationConfig {
    /// Maximum number of Rayon workers used for chunk generation.
    ///
    /// When omitted, Pumpkin chooses a conservative value from the logical CPU count.
    pub max_threads: Option<NonZeroUsize>,
}

impl GenerationConfig {
    /// Resolves the configured worker count for a host reporting `available_cpus`.
    #[must_use]
    pub fn resolve_threads(&self, available_cpus: usize) -> usize {
        self.max_threads.map_or_else(
            || default_generation_threads(available_cpus),
            NonZeroUsize::get,
        )
    }
}

/// Chooses a low-core-aware default while leaving some scheduling headroom.
#[must_use]
pub const fn default_generation_threads(available_cpus: usize) -> usize {
    match available_cpus {
        0 | 1 => 1,
        2..=4 => available_cpus - 1,
        _ => available_cpus - 2,
    }
}

/// Configuration for world and level-specific settings.
///
/// Currently, it includes chunk-related options; more settings may be added later.
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct LevelConfig {
    /// Configuration for chunk behaviour and management.
    pub chunk: ChunkConfig,
    #[serde(default)]
    pub lighting: LightingEngineConfig,
    /// Background chunk-generation worker configuration.
    #[serde(default)]
    pub generation: GenerationConfig,
    /// Number of ticks between autosave checks. If 0, autosave is disabled.
    #[serde(default = "default_autosave_ticks")]
    pub autosave_ticks: u64,
    // TODO: More options
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}

#[cfg(test)]
mod tests {
    use super::{GenerationConfig, default_generation_threads};
    use std::num::NonZeroUsize;

    #[test]
    fn automatic_generation_threads_preserve_low_core_capacity() {
        for (cpus, expected) in [(0, 1), (1, 1), (2, 1), (3, 2), (4, 3), (8, 6), (16, 14)] {
            assert_eq!(default_generation_threads(cpus), expected);
        }
    }

    #[test]
    fn explicit_generation_threads_override_automatic_default() {
        let config = GenerationConfig {
            max_threads: NonZeroUsize::new(7),
        };
        assert_eq!(config.resolve_threads(2), 7);
    }

    #[test]
    fn generation_config_toml_supports_automatic_and_explicit_modes() {
        let automatic: GenerationConfig = toml::from_str("").unwrap();
        assert!(automatic.max_threads.is_none());

        let explicit: GenerationConfig = toml::from_str("max_threads = 7").unwrap();
        assert_eq!(explicit.max_threads.unwrap().get(), 7);
        let serialized = toml::to_string(&explicit).unwrap();
        assert!(serialized.contains("max_threads = 7"));
    }

    #[test]
    fn generation_config_rejects_zero_threads() {
        assert!(toml::from_str::<GenerationConfig>("max_threads = 0").is_err());
    }
}
