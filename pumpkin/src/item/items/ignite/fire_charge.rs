use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::entity::player::Player;
use crate::item::items::ignite::ignition::Ignition;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;

pub struct FireChargeItem;

impl ItemMetadata for FireChargeItem {
    fn ids() -> Box<[u16]> {
        [Item::FIRE_CHARGE.id].into()
    }
}

impl ItemBehaviour for FireChargeItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let ignited = Ignition::ignite_block(
                |world: Arc<World>, pos: BlockPos, new_state_id: BlockStateId| async move {
                    world
                        .set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL)
                        .await;
                },
                player,
                location,
                face,
                block,
            )
            .await;

            if ignited {
                let sound_pos = location.offset(face.to_offset());
                player.world().play_block_sound(
                    Sound::ItemFirechargeUse,
                    SoundCategory::Blocks,
                    sound_pos,
                );
                item.decrement_unless_creative(player.gamemode.load(), 1);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
