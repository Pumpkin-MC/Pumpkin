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
    /// EXPERIMENTAL: offload block-light propagation to a GPU compute shader.
    /// Needs the `gpu-experimental` Cargo feature, and is not production ready.
    #[serde(default)]
    pub gpu_experimental_lighting: bool,
    /// EXPERIMENTAL: offload terrain density noise to a GPU compute shader.
    /// Needs the `gpu-experimental` Cargo feature, and is not production ready.
    #[serde(default)]
    pub gpu_experimental_terrain_noise: bool,
    /// Number of ticks between autosave checks. If 0, autosave is disabled.
    #[serde(default = "default_autosave_ticks")]
    pub autosave_ticks: u64,
    // TODO: More options
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}
