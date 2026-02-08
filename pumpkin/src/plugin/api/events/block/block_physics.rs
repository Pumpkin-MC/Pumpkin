use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block physics update is triggered.
///
/// Block physics updates occur when a neighboring block changes,
/// causing the block to check if it can still exist in its current state.
///
/// If the event is cancelled, the physics update will not occur.
///
/// Matches Bukkit's `BlockPhysicsEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPhysicsEvent {
    /// The block receiving the physics update.
    pub block: &'static Block,

    /// The position of the block.
    pub block_position: BlockPos,

    /// The block that caused the physics update (the changed neighbor).
    pub source_block: &'static Block,

    /// The position of the source block.
    pub source_position: BlockPos,
}

impl BlockPhysicsEvent {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        source_block: &'static Block,
        source_position: BlockPos,
    ) -> Self {
        Self {
            block,
            block_position,
            source_block,
            source_position,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockPhysicsEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
