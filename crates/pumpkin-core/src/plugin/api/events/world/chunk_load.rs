use crate::world::World;
use pumpkin_macros::Event;
use pumpkin_world::level::SyncChunk;
use std::sync::Arc;

/// A notification that a chunk became available in a world.
///
/// Delivered on the world tick after publication. This event cannot be cancelled.
#[derive(Event, Clone)]
pub struct ChunkLoad {
    pub world: Arc<World>,
    pub chunk: SyncChunk,
}
