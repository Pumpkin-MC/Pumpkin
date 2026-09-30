use std::sync::Arc;

use pumpkin_data::damage::DamageType;
use pumpkin_data::enchantment::LevelBasedValue;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro};

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

/// Enchantment entity effect that damages an entity.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageEntity {
    pub min_damage: LevelBasedValue,
    pub max_damage: LevelBasedValue,
    pub damage_type: Option<&'static DamageType>,
}

impl DamageEntity {
    #[must_use]
    pub const fn new(
        min_damage: LevelBasedValue,
        max_damage: LevelBasedValue,
        damage_type: Option<&'static DamageType>,
    ) -> Self {
        Self {
            min_damage,
            max_damage,
            damage_type,
        }
    }

    #[must_use]
    pub fn calculate_damage(&self, level: i32, rng: &mut impl RandomImpl) -> f32 {
        let min = self.min_damage.calculate(level);
        let max = self.max_damage.calculate(level);
        if (max - min).abs() < f32::EPSILON || min >= max {
            min
        } else {
            rng.next_f32().mul_add(max - min, min)
        }
    }

    pub fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
    ) {
        let Some(entity) = entity else {
            return;
        };

        let mut rng = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let damage = self.calculate_damage(enchantment_level, &mut rng);
        let damage_type = self.damage_type.copied().unwrap_or(DamageType::GENERIC);

        let target_entity = world
            .get_player_by_id(entity.entity_id)
            .map(|p| p as Arc<dyn crate::entity::EntityBase>)
            .or_else(|| world.get_entity_by_id(entity.entity_id));

        if let Some(target) = target_entity {
            let caller = owner.map_or_else(
                || target.as_ref(),
                |o| o.as_ref() as &dyn crate::entity::EntityBase,
            );
            target.damage(caller, damage, damage_type);
        }
    }
}

impl EnchantmentEntityEffectExt for DamageEntity {
    fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        _position: Vector3<f64>,
    ) {
        self.apply(world, enchantment_level, owner, entity);
    }
}
