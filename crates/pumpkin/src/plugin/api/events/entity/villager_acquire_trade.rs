use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a villager acquires a new trade.
#[cancellable]
#[derive(Event, Clone)]
pub struct VillagerAcquireTradeEvent {
    /// The ID of the villager entity.
    pub entity_id: i32,

    /// The trade recipe index.
    pub recipe_index: i32,
}
