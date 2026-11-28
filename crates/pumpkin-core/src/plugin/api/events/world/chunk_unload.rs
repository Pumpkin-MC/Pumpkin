use pumpkin_macros::Event;
use pumpkin_util::math::vector2::Vector2;

/// A notification that a chunk was removed from the loaded chunk map.
///
/// Delivered on the world tick after removal. This event cannot be cancelled.
#[derive(Event, Clone)]
pub struct ChunkUnloadEvent {
    /// Chunk coordinates.
    pub chunk_pos: Vector2<i32>,
}

impl ChunkUnloadEvent {
    #[must_use]
    pub const fn new(chunk_pos: Vector2<i32>) -> Self {
        Self { chunk_pos }
    }
}
