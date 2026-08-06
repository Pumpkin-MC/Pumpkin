use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a block fades or melts away (e.g. ice, snow).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFadeEvent {
    /// The position of the fading block.
    pub block_pos: BlockPos,

    /// The block fading.
    pub block: &'static Block,
}

impl BlockFadeEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, block: &'static Block) -> Self {
        Self {
            block_pos,
            block,
            cancelled: false,
        }
    }
}
