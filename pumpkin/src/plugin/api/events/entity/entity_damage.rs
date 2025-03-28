use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::{Entity, EntityBase};
use pumpkin_data::damage::DamageType;

/// An event that occurs when an entity takes damage.
///
/// If the event is cancelled, the entity will not take damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageEvent {
    /// The entity ID that was damaged.
    pub entity: Arc<dyn EntityBase>,

    /// The amount of damage dealt.
    pub damage: f32,

    /// The type of damage dealt.
    pub damage_type: DamageType,
}

impl EntityDamageEvent {
    /// Creates a new instance of `EntityDamageEvent`.
    ///
    /// # Arguments
    /// - `entity`: A reference to the entity that was damaged.
    /// - `damage`: The amount of damage dealt.
    /// - `damage_type`: The type of damage dealt.
    ///
    /// # Returns
    /// A new instance of `EntityDamageEvent`.
    pub fn new(entity: Arc<dyn EntityBase>, damage: f32, damage_type: DamageType) -> Self {
        Self {
            entity,
            damage,
            damage_type,
            cancelled: false,
        }
    }

    /// Gets a reference of the Entity that was Damaged
    ///
    /// # Returns
    // Reference of the Entity
    #[must_use]
    pub fn get_entity(&self) -> &Entity {
        self.entity.get_entity()
    }

    /// Gets the amount of damage dealt.
    ///
    /// # Returns
    /// The amount of damage dealt.
    #[must_use]
    pub fn get_damage(&self) -> f32 {
        self.damage
    }

    /// Sets the amount of damage to be dealt.
    ///
    /// # Arguments
    /// - `damage`: The new amount of damage to be dealt.
    pub fn set_damage(&mut self, damage: f32) {
        self.damage = damage;
    }

    /// Gets the type of damage dealt.
    ///
    /// # Returns
    /// The type of damage dealt.
    #[must_use]
    pub fn get_damage_type(&self) -> DamageType {
        self.damage_type
    }
}
