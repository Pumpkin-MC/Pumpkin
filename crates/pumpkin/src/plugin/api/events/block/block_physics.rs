use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a block physics check is run.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPhysicsEvent {
    pub block_pos: BlockPos,
    pub changed_pos: BlockPos,
}

impl BlockPhysicsEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, changed_pos: BlockPos) -> Self {
        Self {
            block_pos,
            changed_pos,
            cancelled: false,
        }
    }
}
