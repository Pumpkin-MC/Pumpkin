use std::sync::Arc;

use crate::{server::Server, world::portal::end::EndPortal};
use pumpkin_data::{Block, BlockDirection, item::Item};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::{entity::player::Player, world::World};
use async_trait::async_trait;

pub struct EnderEyeItem;

impl ItemMetadata for EnderEyeItem {
    fn ids() -> Box<[u16]> {
        [Item::ENDER_EYE.id].into()
    }
}

#[async_trait]
impl PumpkinItem for EnderEyeItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        if block.id != Block::END_PORTAL_FRAME.id {
            return;
        }

        let world = player.world().await;
        let state_id = world.get_block_state_id(&location).await;
        let original_props = block.properties(state_id).unwrap().to_props();

        let props = original_props
            .iter()
            .map(|(key, value)| {
                if key == "eye" {
                    (key.as_str(), "true")
                } else {
                    (key.as_str(), value.as_str())
                }
            })
            .collect();

        let new_state_id = block.from_properties(props).unwrap().to_state_id(block);
        world
            .set_block_state(&location, new_state_id, BlockFlags::empty())
            .await;

        EndPortal::get_new_portal(&world, location).await;

        return;
    }

    async fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world().await;
        let (start_pos, end_pos) = self.get_start_and_end_pos(player);
        let checker = async |pos: &BlockPos, world_inner: &Arc<World>| {
            let state_id = world_inner.get_block_state_id(pos).await;
            state_id != Block::AIR.default_state.id
        };

        let Some((block_pos, _direction)) = world.raycast(start_pos, end_pos, checker).await else {
            return;
        };

        let (block, _) = world.get_block_and_block_state(&block_pos).await;

        if block.id == Block::END_PORTAL_FRAME.id {
            return;
        }
        //TODO Throw the Ender Eye in the direction of the stronghold.
    }
}
