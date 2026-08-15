use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a villager's reputation changes for a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct VillagerReputationChangeEvent {
    /// Villager entity ID.
    pub entity_id: i32,
    /// Target player entity ID.
    pub target_id: i32,
    /// Reputation change amount.
    pub reputation_change: i32,
}

impl VillagerReputationChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: i32, reputation_change: i32) -> Self {
        Self {
            entity_id,
            target_id,
            reputation_change,
            cancelled: false,
        }
    }
}
