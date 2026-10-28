use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an item is moved between inventories.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryMoveItemEvent {
    /// The source inventory position.
    pub source_pos: BlockPos,

    /// The destination inventory position.
    pub target_pos: BlockPos,

    /// The registry key of the item moved.
    pub item_id: String,

    /// The quantity of item moved.
    pub item_amount: u32,
}

impl InventoryMoveItemEvent {
    #[must_use]
    pub const fn new(
        source_pos: BlockPos,
        target_pos: BlockPos,
        item_id: String,
        item_amount: u32,
    ) -> Self {
        Self {
            source_pos,
            target_pos,
            item_id,
            item_amount,
            cancelled: false,
        }
    }
}
