use pumpkin_data::damage::DamageType;
use pumpkin_macros::Event;

use super::EntityEvent;

/// An event that occurs when an entity dies.
#[derive(Event, Clone)]
pub struct EntityDeathEvent {
    /// The UUID of the entity that died.
    pub entity_uuid: uuid::Uuid,

    /// The damage type that caused the death.
    pub damage_type: DamageType,
}

impl EntityDeathEvent {
    #[must_use]
    pub const fn new(entity_uuid: uuid::Uuid, damage_type: DamageType) -> Self {
        Self {
            entity_uuid,
            damage_type,
        }
    }
}

impl EntityEvent for EntityDeathEvent {
    fn get_entity_uuid(&self) -> uuid::Uuid {
        self.entity_uuid
    }
}
