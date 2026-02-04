use pumpkin_data::damage::DamageType;
use pumpkin_macros::{Event, cancellable};

use super::EntityEvent;

/// An event that occurs when an entity takes damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageEvent {
    /// The UUID of the entity that was damaged.
    pub entity_uuid: uuid::Uuid,

    /// The amount of damage being applied.
    pub damage: f32,

    /// The type of damage.
    pub damage_type: DamageType,
}

impl EntityDamageEvent {
    #[must_use]
    pub const fn new(entity_uuid: uuid::Uuid, damage: f32, damage_type: DamageType) -> Self {
        Self {
            entity_uuid,
            damage,
            damage_type,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityDamageEvent {
    fn get_entity_uuid(&self) -> uuid::Uuid {
        self.entity_uuid
    }
}
