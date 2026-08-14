use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity enters a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityEnterBlockEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// Position of the block entered.
    pub block_pos: BlockPos,
}

impl EntityEnterBlockEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: BlockPos) -> Self {
        Self {
            entity_id,
            block_pos,
            cancelled: false,
        }
    }
}
