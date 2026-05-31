use serde::{Deserialize, Serialize};

use crate::{chunk::ChunkConfig, lighting::LightingEngineConfig};

/// Configuration for world and level-specific settings.
///
/// Currently, it includes chunk-related options; more settings may be added later.
#[derive(Deserialize, Serialize, Default, Clone)]
pub struct LevelConfig {
    /// Configuration for chunk behaviour and management.
    pub chunk: ChunkConfig,
    #[serde(default)]
    pub lighting: LightingEngineConfig,
    /// Number of ticks between autosave checks. If 0, autosave is disabled.
    #[serde(default = "default_autosave_ticks")]
    pub autosave_ticks: u64,
    /// The world generator to use, mirroring the vanilla `level-type` in
    /// `server.properties`. Accepts the vanilla values (the `minecraft:`
    /// prefix is optional), e.g. `minecraft:normal` or `minecraft:flat`.
    /// Unknown values fall back to normal generation.
    #[serde(default = "default_level_type")]
    pub level_type: String,
    /// Generator-specific settings, mirroring the vanilla `generator-settings`
    /// in `server.properties`. For `minecraft:flat` this is the superflat
    /// preset string (layers bottom-up as `count*block`, optionally followed by
    /// `;biome`); an empty value selects the Classic Flat preset.
    #[serde(default)]
    pub generator_settings: String,
    // TODO: More options
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}

fn default_level_type() -> String {
    "minecraft:normal".to_string()
}
