use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};

use super::EntityEvent;

/// An event that occurs when an entity's food level changes.
///
/// If the event is cancelled, the food level change will not occur.
///
/// Matches Bukkit's `FoodLevelChangeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct FoodLevelChangeEvent {
    /// The unique ID of the entity whose food level is changing.
    pub entity_id: i32,

    /// The type of entity (typically a player).
    pub entity_type: &'static EntityType,

    /// The new food level.
    pub food_level: i32,
}

impl FoodLevelChangeEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        food_level: i32,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            food_level,
            cancelled: false,
        }
    }
}

impl EntityEvent for FoodLevelChangeEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
