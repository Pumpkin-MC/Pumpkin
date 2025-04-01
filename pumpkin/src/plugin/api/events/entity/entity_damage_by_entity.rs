use std::sync::Arc;

use pumpkin_macros::{Event, cancellable};

use crate::entity::{Entity, EntityBase};
use pumpkin_data::damage::DamageType;

use super::entity_damage::EntityDamageEvent;

/// An event that occurs when an entity is damaged by another entity.
///
/// If the event is cancelled, the entity will not take damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageByEntityEvent {
    /// The base damage event.
    pub base_event: EntityDamageEvent,

    /// The entity ID that caused the damage.
    pub attacker: Arc<dyn EntityBase>,
}

impl EntityDamageByEntityEvent {
    /// Creates a new instance of `EntityDamageByEntityEvent`.
    ///
    /// # Arguments
    /// - `victim`: A reference to the entity that was damaged.
    /// - `attacker`: A reference to the entity that caused the damage.
    /// - `damage`: The amount of damage dealt.
    /// - `damage_type`: The type of damage dealt.
    ///
    /// # Returns
    /// A new instance of `EntityDamageByEntityEvent`.
    pub fn new(
        victim: Arc<dyn EntityBase>,
        attacker: Arc<dyn EntityBase>,
        damage: f32,
        damage_type: DamageType,
    ) -> Self {
        Self {
            base_event: EntityDamageEvent::new(victim, damage, damage_type),
            attacker: attacker as Arc<dyn EntityBase>,
            cancelled: false,
        }
    }

    /// Gets the entity that caused the damage.
    ///
    /// # Returns
    /// A reference to the entity that caused the damage.
    #[must_use]
    pub fn get_attacker(&self) -> &Entity {
        self.attacker.get_entity()
    }

    /// Returns the base `EntityDamageEvent`
    ///
    /// # Returns
    /// The base `EntityDamageEvent`
    #[must_use]
    pub fn get_base_event(&self) -> &EntityDamageEvent {
        &self.base_event
    }
}
