use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block fades, melts, or decays.
///
/// This covers ice melting, snow melting, coral dying,
/// farmland drying, leaves decaying, etc.
///
/// If the event is cancelled, the fade will not occur.
///
/// Matches Bukkit's `BlockFadeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFadeEvent {
    /// The block that is fading.
    pub block: &'static Block,

    /// The position of the block.
    pub block_position: BlockPos,

    /// The new block state after fading (typically air).
    pub new_block: &'static Block,
}

impl BlockFadeEvent {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        new_block: &'static Block,
    ) -> Self {
        Self {
            block,
            block_position,
            new_block,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockFadeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
