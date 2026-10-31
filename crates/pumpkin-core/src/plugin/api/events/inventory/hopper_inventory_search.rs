use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a hopper searches for a container inventory.
#[cancellable]
#[derive(Event, Clone)]
pub struct HopperInventorySearchEvent {
    /// The block position of the hopper.
    pub block_pos: BlockPos,

    /// The target search position.
    pub search_pos: BlockPos,
}

impl HopperInventorySearchEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, search_pos: BlockPos) -> Self {
        Self {
            block_pos,
            search_pos,
            cancelled: false,
        }
    }
}
