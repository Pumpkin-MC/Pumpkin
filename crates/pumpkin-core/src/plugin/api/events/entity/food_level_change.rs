use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity's food level changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct FoodLevelChangeEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// The new food level.
    pub food_level: u8,
}

impl FoodLevelChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, food_level: u8) -> Self {
        Self {
            entity_id,
            food_level,
            cancelled: false,
        }
    }
}
