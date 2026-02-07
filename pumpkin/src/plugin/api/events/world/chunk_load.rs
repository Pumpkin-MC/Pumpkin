use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::chunk::ChunkData;
use std::sync::Arc;

/// An event that occurs when a chunk is loaded in a world.
///
/// This event contains information about the world and the chunk being loaded.
#[cancellable]
#[derive(Event, Clone)]
pub struct ChunkLoad {
    /// The world in which the chunk is being loaded.
    pub world: Arc<World>,

    /// The chunk data being loaded.
    pub chunk: Arc<ChunkData>,
}
