use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity's remaining air supply changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityAirChangeEvent {
    /// The ID of the entity.
    pub entity_id: i32,

    /// The new air amount.
    pub amount: i32,
}

impl EntityAirChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, amount: i32) -> Self {
        Self {
            entity_id,
            amount,
            cancelled: false,
        }
    }
}
