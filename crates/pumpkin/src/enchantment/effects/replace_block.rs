use std::sync::Arc;

use pumpkin_data::game_event::GameEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

/// Enchantment entity effect that replaces a block at an offset position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReplaceBlock {
    pub offset: Vector3<i32>,
    pub trigger_game_event: Option<GameEvent>,
}

impl ReplaceBlock {
    #[must_use]
    pub const fn new(offset: Vector3<i32>, trigger_game_event: Option<GameEvent>) -> Self {
        Self {
            offset,
            trigger_game_event,
        }
    }

    #[must_use]
    pub const fn target_position(&self, origin: Vector3<f64>) -> BlockPos {
        let base_x = origin.x.floor() as i32;
        let base_y = origin.y.floor() as i32;
        let base_z = origin.z.floor() as i32;
        BlockPos::new(
            base_x + self.offset.x,
            base_y + self.offset.y,
            base_z + self.offset.z,
        )
    }
}

impl EnchantmentEntityEffectExt for ReplaceBlock {
    fn apply(
        &self,
        world: &Arc<World>,
        _enchantment_level: i32,
        _owner: Option<&Arc<Player>>,
        _entity: Option<&Entity>,
        position: Vector3<f64>,
    ) {
        let target = self.target_position(position);
        if let Some(event) = self.trigger_game_event {
            world.emit_game_event(event.name(), target.to_centered_f64());
        }
    }
}
