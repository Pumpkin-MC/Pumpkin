use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a block dispenses an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDispenseEvent {
    pub block_pos: BlockPos,
    pub item_name: String,
}

impl BlockDispenseEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, item_name: String) -> Self {
        Self {
            block_pos,
            item_name,
            cancelled: false,
        }
    }
}
