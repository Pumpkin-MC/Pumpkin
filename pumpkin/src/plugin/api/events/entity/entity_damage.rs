use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::world::World;

use super::EntityEvent;

/// An event that occurs when an entity takes damage.
///
/// If the event is cancelled, the damage will not be applied.
///
/// This event contains information about the entity taking damage,
/// the damage amount, the damage type, and the world the entity is in.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageEvent {
    /// The unique ID of the entity taking damage.
    pub entity_id: i32,

    /// The type of entity taking damage.
    pub entity_type: &'static EntityType,

    /// The amount of damage being dealt.
    pub damage: f32,

    /// The type of damage being dealt.
    pub damage_type: &'static DamageType,

    /// The world in which the damage is occurring.
    pub world: Arc<World>,
}

impl EntityDamageEvent {
    /// Creates a new instance of `EntityDamageEvent`.
    #[must_use]
    pub fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        damage: f32,
        damage_type: &'static DamageType,
        world: Arc<World>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            damage,
            damage_type,
            world,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityDamageEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
