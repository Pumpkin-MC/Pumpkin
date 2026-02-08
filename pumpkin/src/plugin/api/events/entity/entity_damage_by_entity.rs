use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::world::World;

use super::EntityEvent;

/// An event that occurs when an entity is damaged by another entity.
///
/// If the event is cancelled, the damage will not be applied.
///
/// This is a more specific version of `EntityDamageEvent` that includes
/// information about the attacking entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageByEntityEvent {
    /// The unique ID of the entity taking damage.
    pub entity_id: i32,

    /// The type of entity taking damage.
    pub entity_type: &'static EntityType,

    /// The unique ID of the attacking entity.
    pub attacker_id: i32,

    /// The type of the attacking entity.
    pub attacker_type: &'static EntityType,

    /// The amount of damage being dealt.
    pub damage: f32,

    /// The type of damage being dealt.
    pub damage_type: &'static DamageType,

    /// The world in which the damage is occurring.
    pub world: Arc<World>,
}

impl EntityDamageByEntityEvent {
    /// Creates a new instance of `EntityDamageByEntityEvent`.
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        attacker_id: i32,
        attacker_type: &'static EntityType,
        damage: f32,
        damage_type: &'static DamageType,
        world: Arc<World>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            attacker_id,
            attacker_type,
            damage,
            damage_type,
            world,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityDamageByEntityEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
