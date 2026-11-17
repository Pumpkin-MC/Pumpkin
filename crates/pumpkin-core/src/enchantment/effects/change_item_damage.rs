use std::sync::Arc;

use pumpkin_data::enchantment::LevelBasedValue;
use pumpkin_util::math::vector3::Vector3;

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

/// Enchantment entity effect that alters item damage.
#[derive(Clone, Debug, PartialEq)]
pub struct ChangeItemDamage {
    pub amount: LevelBasedValue,
}

impl ChangeItemDamage {
    #[must_use]
    pub const fn new(amount: LevelBasedValue) -> Self {
        Self { amount }
    }

    pub fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
    ) {
        let player = owner
            .cloned()
            .or_else(|| entity.and_then(|e| world.get_player_by_id(e.entity_id)));

        if let Some(player) = player {
            let change = self.amount.calculate(enchantment_level) as i32;
            player.damage_held_item(change);
        }
    }
}

impl EnchantmentEntityEffectExt for ChangeItemDamage {
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
