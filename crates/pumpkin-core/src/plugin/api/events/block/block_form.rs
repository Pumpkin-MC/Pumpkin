use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a block forms naturally (e.g. ice, snow, cobblestone).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFormEvent {
    /// The position where the block forms.
    pub block_pos: BlockPos,

    /// The formed block type.
    pub block: &'static Block,
}

impl BlockFormEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, block: &'static Block) -> Self {
        Self {
            block_pos,
            block,
            cancelled: false,
        }
    }
}
