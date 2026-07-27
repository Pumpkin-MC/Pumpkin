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
    /// Number of threads used for chunk generation (the shared rayon pool).
    ///
    /// `0` (default) means auto: `max(1, cores - 2)`. The generation pool is
    /// not the only thing running — the tokio runtime spawns one network/IO
    /// worker per core, and each dimension runs its own scheduler thread — so
    /// sizing the pool to the full core count oversubscribes the machine and
    /// pushes total CPU usage past `cores * 100%`. Leaving two cores of
    /// headroom keeps networking and ticking responsive while chunks generate.
    ///
    /// Lower this further on thermally constrained hosts (e.g. phones or SBCs)
    /// to cap generation CPU usage.
    #[serde(default)]
    pub chunk_generation_threads: usize,
    // TODO: More options
}

impl LevelConfig {
    /// Resolves `chunk_generation_threads`, applying the auto default
    /// (`max(1, cores - 2)`) when the configured value is `0`.
    #[must_use]
    pub fn resolved_chunk_generation_threads(&self) -> usize {
        if self.chunk_generation_threads > 0 {
            return self.chunk_generation_threads;
        }
        std::thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .saturating_sub(2)
            .max(1)
    }
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}
