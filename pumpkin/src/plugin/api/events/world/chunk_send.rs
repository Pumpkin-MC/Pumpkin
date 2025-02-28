use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::chunk::ChunkData;
use std::sync::Arc;
use tokio::sync::RwLock;

/// An event that occurs when a chunk is sent to a client.
///
/// This event contains information about the world and the chunk being sent.
#[cancellable]
#[derive(Event, Clone)]
pub struct ChunkSend {
    /// The world from which the chunk is being sent.
    pub world: Arc<World>,

    /// The chunk data being sent, wrapped in a read-write lock for safe concurrent access.
    pub chunk: Arc<RwLock<ChunkData>>,
}
