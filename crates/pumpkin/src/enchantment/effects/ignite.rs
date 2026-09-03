use std::sync::Arc;

use pumpkin_data::enchantment::LevelBasedValue;
use pumpkin_util::math::vector3::Vector3;

use super::EnchantmentEntityEffectExt;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

/// Enchantment entity effect that sets an entity on fire for a duration calculated from the level.
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
    pub fn apply_to_entity(&self, level: i32, entity: &Entity) {
        let seconds = self.duration.calculate(level);
        entity.set_on_fire_for(seconds);
        entity.set_on_fire(true);
    }
}

impl EnchantmentEntityEffectExt for Ignite {
    fn apply(
        &self,
        _world: &Arc<World>,
        enchantment_level: i32,
        _owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        _position: Vector3<f64>,
    ) {
        if let Some(entity) = entity {
            self.apply_to_entity(enchantment_level, entity);
        }
    }
}
