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
    /// Needs the `gpu-experimental-lighting` Cargo feature, and is not production
    /// ready. Only pays off for bulk relighting of a large region held resident on a
    /// discrete GPU; it lost to the CPU on integrated graphics at every size tested.
    #[serde(default)]
    pub gpu_experimental_lighting: bool,
    /// EXPERIMENTAL: offload terrain density noise to a GPU compute shader.
    /// Needs the `gpu-experimental-noise` Cargo feature, and is not production ready.
    /// Only pays off in bulk, roughly nine chunks or more per dispatch; generating a
    /// single chunk is much slower on the GPU than on the CPU.
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
