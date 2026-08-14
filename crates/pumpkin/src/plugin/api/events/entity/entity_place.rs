use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity places a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityPlaceEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// Position of the placed block.
    pub block_pos: BlockPos,
    /// The placed block state identifier.
    pub block_name: String,
}

impl EntityPlaceEvent {
    #[must_use]
    pub const fn new(entity_id: i32, block_pos: BlockPos, block_name: String) -> Self {
        Self {
            entity_id,
            block_pos,
            block_name,
            cancelled: false,
        }
    }
}
