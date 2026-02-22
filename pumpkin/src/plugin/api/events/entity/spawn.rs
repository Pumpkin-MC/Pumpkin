use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};

use super::EntityEvent;

/// An event that occurs when an entity is spawned.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntitySpawnEvent {
    /// The UUID of the entity being spawned.
    pub entity_uuid: uuid::Uuid,

    /// The type of the entity being spawned.
    pub entity_type: &'static EntityType,
}

impl EntitySpawnEvent {
    #[must_use]
    pub const fn new(entity_uuid: uuid::Uuid, entity_type: &'static EntityType) -> Self {
        Self {
            entity_uuid,
            entity_type,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntitySpawnEvent {
    fn get_entity_uuid(&self) -> uuid::Uuid {
        self.entity_uuid
    }
}
