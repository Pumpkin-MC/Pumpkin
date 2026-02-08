use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};

use super::EntityEvent;

/// An event that occurs when an entity targets another entity.
///
/// If the event is cancelled, the targeting will not occur.
///
/// Matches Bukkit's `EntityTargetEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTargetEvent {
    /// The unique ID of the entity doing the targeting.
    pub entity_id: i32,

    /// The type of entity doing the targeting.
    pub entity_type: &'static EntityType,

    /// The unique ID of the target entity, or `None` if losing a target.
    pub target_id: Option<i32>,
}

impl EntityTargetEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        target_id: Option<i32>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            target_id,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityTargetEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
