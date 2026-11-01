use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a block explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockExplodeEvent {
    pub block_pos: BlockPos,
    pub yield_rate: f32,
}

impl BlockExplodeEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, yield_rate: f32) -> Self {
        Self {
            block_pos,
            yield_rate,
            cancelled: false,
        }
    }
}
