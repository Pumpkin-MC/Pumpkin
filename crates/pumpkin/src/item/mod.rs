pub mod items;
pub mod potion;
pub mod registry;

use std::any::Any;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub trait ItemMetadata {
    fn ids() -> Box<[u16]>;
}

pub trait ItemBehaviour: Send + Sync {
    fn normal_use(&self, _item: &Item, _player: &Player) {}

    #[expect(clippy::too_many_arguments)]
    fn use_on_block(
        &self,
        _item: &mut ItemStack,
        _player: &Player,
        _location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) {
    }

    fn use_on_entity(&self, _item: &mut ItemStack, _player: &Player, _entity: Arc<dyn EntityBase>) {
    }

    fn on_stopped_using(&self, _stack: &ItemStack, _player: &Player) {}

    fn on_spear_jab(&self, _stack: &ItemStack, _player: &Player) {}

    fn on_use_tick(&self, _stack: &ItemStack, _player: &Player, _remaining_use_ticks: i32) {}

    /// Returns the maximum number of ticks this item can be used for.
    /// Return 0 if the item does not have a behaviour-driven use duration.
    fn get_use_duration(&self) -> i32 {
        0
    }

    fn can_mine(&self, _player: &Player) -> bool {
        true
    }

    fn get_start_and_end_pos(&self, player: &Player) -> (Vector3<f64>, Vector3<f64>) {
        let start_pos = player.eye_position();
        let (yaw, pitch) = player.rotation();
        let (yaw_rad, pitch_rad) = (f64::from(yaw.to_radians()), f64::from(pitch.to_radians()));
        let block_interaction_range = 4.5; // This is not the same as the block_interaction_range in the
        // player entity.
        let direction = Vector3::new(
            -yaw_rad.sin() * pitch_rad.cos() * block_interaction_range,
            -pitch_rad.sin() * block_interaction_range,
            pitch_rad.cos() * yaw_rad.cos() * block_interaction_range,
        );

        let end_pos = start_pos.add(&direction);
        (start_pos, end_pos)
    }

    fn as_any(&self) -> &dyn Any;
}
