use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a villager replenishes its trades.
#[cancellable]
#[derive(Event, Clone)]
pub struct VillagerReplenishTradeEvent {
    /// The ID of the villager entity.
    pub entity_id: i32,

    /// The number of times the trade was restocked.
    pub restock_quantity: i32,
}
