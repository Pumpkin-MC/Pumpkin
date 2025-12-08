pub mod items;
pub mod registry;

use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

pub trait ItemMetadata {
    fn ids() -> Box<[u16]>;
}

pub trait ItemBehaviour: Send + Sync {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        _player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        _player: &'a Player,
        _location: BlockPos,
        _face: BlockDirection,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn use_on_entity<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        _player: &'a Player,
        _entity: Arc<dyn EntityBase>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
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
