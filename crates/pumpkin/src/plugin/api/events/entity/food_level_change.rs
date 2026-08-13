use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a player's food level changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct FoodLevelChangeEvent {
    /// The ID of the player entity whose food level is changing.
    pub entity_id: i32,

    /// The resultant food level that should be applied.
    pub food_level: i32,

    /// The registry name of the item that triggered the change, if any.
    pub item_name: Option<String>,
}

impl FoodLevelChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, food_level: i32, item_name: Option<String>) -> Self {
        Self {
            entity_id,
            food_level,
            item_name,
            cancelled: false,
        }
    }
}
