use std::pin::Pin;
use std::sync::Arc;

use crate::{server::Server, world::portal::end::EndPortal};
use pumpkin_data::{Block, BlockDirection, item::Item};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;

use crate::item::{ItemBehaviour, ItemMetadata};
use crate::{entity::player::Player, world::World};

pub struct EnderEyeItem;

impl ItemMetadata for EnderEyeItem {
    fn ids() -> Box<[u16]> {
        [Item::ENDER_EYE.id].into()
    }
}

impl ItemBehaviour for EnderEyeItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if block.id != Block::END_PORTAL_FRAME.id {
                return;
            }

            let world = player.world();
            let state_id = world.get_block_state_id(&location).await;
            let original_props = block.properties(state_id).unwrap().to_props();

            let props: Vec<(&str, &str)> = original_props
                .iter()
                .map(|(key, value)| {
                    if *key == "eye" {
                        (*key, "true")
                    } else {
                        (*key, *value)
                    }
                })
                .collect();

            let new_state_id = block.from_properties(&props).to_state_id(block);
            world
                .set_block_state(&location, new_state_id, BlockFlags::empty())
                .await;

            EndPortal::get_new_portal(world, location).await;
        })
    }

    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let (start_pos, end_pos) = self.get_start_and_end_pos(player);
            let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
                let state_id = world_inner.get_block_state_id(pos).await;
                state_id != Block::AIR.default_state.id
            };

            let Some((block_pos, _direction)) = world.raycast(start_pos, end_pos, checker).await
            else {
                return;
            };

            let block = world.get_block(&block_pos).await;

            if block == &Block::END_PORTAL_FRAME {}
        })
        //TODO Throw the Ender Eye in the direction of the stronghold.
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
