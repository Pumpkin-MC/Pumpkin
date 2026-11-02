use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a sponge absorbs water.
#[cancellable]
#[derive(Event, Clone)]
pub struct SpongeAbsorbEvent {
    pub block_pos: BlockPos,
}

impl SpongeAbsorbEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos) -> Self {
        Self {
            block_pos,
            cancelled: false,
        }
    }
}
