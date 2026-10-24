use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a furnace smelts an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct FurnaceSmeltEvent {
    /// The position of the furnace block.
    pub block_pos: BlockPos,

    /// The source item being smelted.
    pub source_item: String,

    /// The resulting smelted item.
    pub result_item: String,
}

impl FurnaceSmeltEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, source_item: String, result_item: String) -> Self {
        Self {
            block_pos,
            source_item,
            result_item,
            cancelled: false,
        }
    }
}
