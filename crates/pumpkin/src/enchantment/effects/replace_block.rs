use pumpkin_data::game_event::GameEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// Enchantment entity effect that replaces a block at an offset position.
/// Matches vanilla `net.minecraft.world.item.enchantment.effects.ReplaceBlock`.
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
