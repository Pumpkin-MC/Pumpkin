use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a potion status effect is applied to an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityPotionEffectEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// Effect name.
    pub effect_name: String,
    /// Duration in ticks.
    pub duration: i32,
    /// Amplifier.
    pub amplifier: u8,
}

impl EntityPotionEffectEvent {
    #[must_use]
    pub const fn new(entity_id: i32, effect_name: String, duration: i32, amplifier: u8) -> Self {
        Self {
            entity_id,
            effect_name,
            duration,
            amplifier,
            cancelled: false,
        }
    }
}
