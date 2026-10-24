use pumpkin_data::damage::DamageType;
use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity takes damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageEvent {
    /// The ID of the entity taking damage.
    pub entity_id: i32,

    /// The amount of damage taken.
    pub damage: f32,

    /// The damage type.
    pub damage_type: DamageType,
}

impl EntityDamageEvent {
    #[must_use]
    pub const fn new(entity_id: i32, damage_type: DamageType, damage_amount: f32) -> Self {
        Self {
            entity_id,
            damage: damage_amount,
            damage_type,
            cancelled: false,
        }
    }
}
