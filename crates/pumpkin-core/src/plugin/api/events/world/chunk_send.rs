use crate::server::Server;
use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::level::SyncChunk;
use std::sync::Arc;

/// An event that occurs when a chunk is sent to a client.
///
/// This event contains information about the world and the chunk being sent.
#[cancellable]
#[derive(Event, Clone)]
pub struct ChunkSend {
    /// The world from which the chunk is being sent.
    pub world: Arc<World>,

    /// The chunk data being sent.
    pub chunk: Arc<ChunkData>,
}

impl ChunkSend {
    pub const fn new(world: Arc<World>, chunk: Arc<ChunkData>) -> Self {
        Self {
            world,
            chunk,
            cancelled: false,
        }
    }

    /// Fires the event per chunk -> returns the chunks no plugin cancelled.
    pub async fn filter(
        server: &Arc<Server>,
        world: &Arc<World>,
        chunks: &[SyncChunk],
    ) -> Vec<SyncChunk> {
        let mut valid_chunks = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let mut event = Self::new(world.clone(), chunk.clone());
            server.plugin_manager.fire(server, &mut event).await;
            if !event.cancelled {
                valid_chunks.push(chunk.clone());
            }
        }
        valid_chunks
    }
}
