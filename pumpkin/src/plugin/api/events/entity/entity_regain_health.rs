use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};

use super::EntityEvent;

/// An event that occurs when an entity regains health.
///
/// If the event is cancelled, the health will not be regained.
///
/// Matches Bukkit's `EntityRegainHealthEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityRegainHealthEvent {
    /// The unique ID of the entity regaining health.
    pub entity_id: i32,

    /// The type of entity regaining health.
    pub entity_type: &'static EntityType,

    /// The amount of health being regained.
    pub amount: f32,

    /// The reason for the health regain (e.g. "natural", "eating", "potion").
    pub reason: String,
}

impl EntityRegainHealthEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        amount: f32,
        reason: String,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            amount,
            reason,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityRegainHealthEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
