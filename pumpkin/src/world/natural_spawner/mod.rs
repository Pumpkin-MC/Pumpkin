//! Natural mob spawning, split into cohesive submodules:
//! - `state`: spawn-cap accounting (`SpawnState`, mob counts, local caps, potentials)
//! - `cycle`: the per-chunk spawn cycle and generation-time pack spawning
//! - `rules`: spawn position validity rules and weighted spawner selection
mod cycle;
mod rules;
mod state;

pub use cycle::*;
pub use rules::*;
pub use state::*;

/// Vanilla `DistanceManager` natural-spawn tracker radius
/// (`FixedPlayerDistanceChunkTracker(8)`). Independent of simulation distance.
pub const NATURAL_SPAWN_CHUNK_RANGE: i32 = 8;

/// Vanilla `NaturalSpawner.SPAWN_DISTANCE_BLOCK` squared
/// (`ChunkMap.playerIsCloseEnoughForSpawning` uses 128² = 16384).
pub const SPAWN_DISTANCE_BLOCK_SQ: f64 = 128.0 * 128.0;
