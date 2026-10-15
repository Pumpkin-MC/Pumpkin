use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector2::Vector2;

/// An event that occurs when entities in a chunk are loaded.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntitiesLoadEvent {
    /// Chunk coordinates.
    pub chunk_pos: Vector2<i32>,
    /// Entity count.
    pub entity_count: usize,
}

impl EntitiesLoadEvent {
    #[must_use]
    pub const fn new(chunk_pos: Vector2<i32>, entity_count: usize) -> Self {
        Self {
            chunk_pos,
            entity_count,
            cancelled: false,
        }
    }
}
