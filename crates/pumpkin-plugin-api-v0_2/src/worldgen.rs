//! Custom chunk generation for the async v0.2 world
//!
//! Register a [`ChunkGenerator`] to get an id, point a world at it, and the host calls
//! `handle-generate-phase` once per phase and chunk
//!
//! The chunk buffer arrives by value, so a generator owns it for the whole phase

use std::sync::Arc;

pub use crate::host::{biomes::Biome, world::GenerationPhase};
use crate::{
    host::world::ChunkBuffer,
    registry::{LocalBoxFuture, Registry},
};

static CHUNK_GENERATORS: Registry<dyn ErasedChunkGenerator> = Registry::new();

/// Custom world generation logic
///
/// Register an implementation with [`register_chunk_generator`]
///
/// The phases run in the order biomes, noise, surface and features, and each one is its own
/// method so you only write the ones you need; every default does nothing
///
/// The phase methods are async because the export is, which leaves room to await other host work,
/// while the buffer methods (`set_block_state_id`, `fill_layer` and friends) are synchronous host
/// calls
///
/// # Examples
///
/// ```rust,ignore
/// struct Flat;
///
/// impl ChunkGenerator for Flat {
///     async fn generate_surface(&self, chunk: ChunkBuffer) {
///         // fill_layer is a plain import; 1 stands in for a real block state id
///         chunk.fill_layer(chunk.get_min_y(), 1);
///     }
/// }
/// ```
#[allow(
    async_fn_in_trait,
    unused_variables,
    reason = "guest futures are intentionally not Send and defaults ignore their arguments"
)]
pub trait ChunkGenerator: Send + Sync + 'static {
    /// Assigns biomes across the chunk column
    async fn generate_biomes(&self, chunk: ChunkBuffer) {}

    /// Generates the basic terrain shape
    async fn generate_noise(&self, chunk: ChunkBuffer) {}

    /// Applies surface rules such as the top layers of grass, sand and stone
    async fn generate_surface(&self, chunk: ChunkBuffer) {}

    /// Places features, structures and decorations
    async fn generate_features(&self, chunk: ChunkBuffer) {}
}

trait ErasedChunkGenerator: Send + Sync {
    fn generate(&self, phase: GenerationPhase, chunk: ChunkBuffer) -> LocalBoxFuture<'_, ()>;
}

impl<G: ChunkGenerator> ErasedChunkGenerator for G {
    fn generate(&self, phase: GenerationPhase, chunk: ChunkBuffer) -> LocalBoxFuture<'_, ()> {
        match phase {
            GenerationPhase::Biomes => Box::pin(self.generate_biomes(chunk)),
            GenerationPhase::Noise => Box::pin(self.generate_noise(chunk)),
            GenerationPhase::Surface => Box::pin(self.generate_surface(chunk)),
            GenerationPhase::Features => Box::pin(self.generate_features(chunk)),
        }
    }
}

/// Registers a generator and returns the id to pass to a world's `set_chunk_generator`
///
/// # Examples
///
/// ```rust,ignore
/// let id = worldgen::register_chunk_generator(Flat);
/// world.set_chunk_generator(id);
/// ```
pub fn register_chunk_generator<G: ChunkGenerator>(generator: G) -> u32 {
    CHUNK_GENERATORS.register(Arc::new(generator))
}

pub(crate) async fn dispatch(generator_id: u32, phase: GenerationPhase, chunk: ChunkBuffer) {
    // The registry lock is released before the generator runs
    if let Some(generator) = CHUNK_GENERATORS.get(generator_id) {
        generator.generate(phase, chunk).await;
    }
}
