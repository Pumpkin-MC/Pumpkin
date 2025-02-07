use crate::world::World;
use pumpkin_macros::{cancellable, Event};
use pumpkin_world::chunk::ChunkData;
use std::sync::Arc;
use tokio::sync::RwLock;

/// An event that occurs when a chunk is loaded in a world.
///
/// This event contains information about the world and the chunk being loaded.
#[cancellable]
#[derive(Event, Clone)]
pub struct ChunkLoad {
    /// The world in which the chunk is being loaded.
    pub world: Arc<World>,

    /// The chunk data being loaded, wrapped in a read-write lock for safe concurrent access.
    pub chunk: Arc<RwLock<ChunkData>>,
}

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

/// An event that occurs when a chunk is saved in a world.
///
/// This event contains information about the world and the chunk being saved.
#[cancellable]
#[derive(Event, Clone)]
pub struct ChunkSave {
    /// The world in which the chunk is being saved.
    pub world: Arc<World>,

    /// The chunk data being saved, wrapped in a read-write lock for safe concurrent access.
    pub chunk: Arc<RwLock<ChunkData>>,
}
