use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a container (like a hopper) picks up an item entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryPickupItemEvent {
    /// The block position of the container.
    pub block_pos: BlockPos,

    /// The entity ID of the picked up item.
    pub item_entity_id: i32,

    /// The registry key of the item.
    pub item_id: String,
}

impl InventoryPickupItemEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, item_entity_id: i32, item_id: String) -> Self {
        Self {
            block_pos,
            item_entity_id,
            item_id,
            cancelled: false,
        }
    }
}
