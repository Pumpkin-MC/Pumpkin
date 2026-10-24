use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a piglin barters.
#[cancellable]
#[derive(Event, Clone)]
pub struct PiglinBarterEvent {
    /// The ID of the piglin entity.
    pub entity_id: i32,

    /// The item given to the piglin.
    pub input_item: ItemStack,

    /// The outcome item stacks produced by the barter.
    pub outcome: Vec<ItemStack>,
}

impl PiglinBarterEvent {
    #[must_use]
    pub const fn new(entity_id: i32, input_item: ItemStack, outcome: Vec<ItemStack>) -> Self {
        Self {
            cancelled: false,
            entity_id,
            input_item,
            outcome,
        }
    }
}
