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
    /// `0` (default) means auto, which mirrors vanilla's background pool.
    ///
    /// Vanilla runs that pool nearly maxed out — `clamp(cores - 1, 1, 255)` in
    /// `Util.maxAllowedExecutorThreads` (`/root/Vanilla/src/net/minecraft/util/Util.java:262`,
    /// cap from `Util.getMaxThreads` at `Util.java:279`) — and does *not*
    /// shrink it to leave headroom. What vanilla limits instead is how many
    /// chunks may be in execution at once
    /// (`/root/Vanilla/src/net/minecraft/server/level/ThrottlingChunkTaskDispatcher.java:42`),
    /// which caps queued work and resident chunk data without capping the
    /// throughput of the workers themselves.
    ///
    /// Set this explicitly to pin the pool smaller on a thermally constrained
    /// host (phones, SBCs). It only caps the pool: the in-execution bound is a
    /// fixed constant, so a pinned pool trades chunk latency for CPU instead of
    /// also starving the queue.
    #[serde(default)]
    pub chunk_generation_threads: usize,
    // TODO: More options
}

/// Upper bound vanilla accepts for its background pool
/// (`/root/Vanilla/src/net/minecraft/util/Util.java:270`, default at `:279`).
const VANILLA_MAX_BG_THREADS: usize = 255;

impl LevelConfig {
    /// Resolves `chunk_generation_threads`, applying vanilla's auto sizing.
    ///
    /// Auto (`0`) is `clamp(cores - 1, 1, 255)`, matching
    /// `Util.maxAllowedExecutorThreads`
    /// (`/root/Vanilla/src/net/minecraft/util/Util.java:262`). An explicit
    /// value is honored verbatim as an operator cap.
    #[must_use]
    pub fn resolved_chunk_generation_threads(&self) -> usize {
        if self.chunk_generation_threads > 0 {
            return self.chunk_generation_threads;
        }
        std::thread::available_parallelism()
            .map_or(1, std::num::NonZero::get)
            .saturating_sub(1)
            .clamp(1, VANILLA_MAX_BG_THREADS)
    }
}

const fn default_autosave_ticks() -> u64 {
    6000 // Default to 5 minutes at 20 TPS
}
