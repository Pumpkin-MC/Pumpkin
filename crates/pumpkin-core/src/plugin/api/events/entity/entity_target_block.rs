use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity targets a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTargetBlockEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// Targeted block position.
    pub block_pos: BlockPos,
}

impl EntityTargetBlockEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: BlockPos) -> Self {
        Self {
            entity_id,
            block_pos,
            cancelled: false,
        }
    }
}
