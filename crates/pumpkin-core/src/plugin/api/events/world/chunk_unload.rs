use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector2::Vector2;

/// An event that occurs when a chunk is unloaded.
#[cancellable]
#[derive(Event, Clone)]
pub struct ChunkUnloadEvent {
    /// Chunk coordinates.
    pub chunk_pos: Vector2<i32>,
}

impl ChunkUnloadEvent {
    #[must_use]
    pub const fn new(chunk_pos: Vector2<i32>) -> Self {
        Self {
            chunk_pos,
            cancelled: false,
        }
    }
}
