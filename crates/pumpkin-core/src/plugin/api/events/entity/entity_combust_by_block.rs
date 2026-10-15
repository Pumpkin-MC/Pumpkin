use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity is set on fire by a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityCombustByBlockEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// Combuster block pos.
    pub combuster: BlockPos,
    /// Combust duration in seconds.
    pub duration: f32,
}

impl EntityCombustByBlockEvent {
    #[must_use]
    pub const fn new(entity_id: i32, combuster: BlockPos, duration: f32) -> Self {
        Self {
            entity_id,
            combuster,
            duration,
            cancelled: false,
        }
    }
}
