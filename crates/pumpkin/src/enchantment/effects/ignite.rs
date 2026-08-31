use crate::entity::{Entity, EntityBase};
use pumpkin_data::enchantment::LevelBasedValue;

/// Enchantment entity effect that sets an entity on fire for a duration calculated from the level.
/// Matches vanilla `net.minecraft.world.item.enchantment.effects.Ignite`.
#[derive(Clone, Debug, PartialEq)]
pub struct Ignite {
    pub duration: LevelBasedValue,
}

impl Ignite {
    #[must_use]
    pub const fn new(duration: LevelBasedValue) -> Self {
        Self { duration }
    }

    /// Applies the ignite effect to an entity for the given enchantment level.
    pub fn apply(&self, level: i32, entity: &Entity) {
        let seconds = self.duration.calculate(level);
        entity.set_on_fire_for(seconds);
        entity.set_on_fire(true);
    }
}
