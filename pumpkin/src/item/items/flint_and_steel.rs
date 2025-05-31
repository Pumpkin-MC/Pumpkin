use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::fire::FireBlockBase;
use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;

pub struct FlintAndSteelItem;

impl ItemMetadata for FlintAndSteelItem {
    fn ids() -> Box<[u16]> {
        [Item::FLINT_AND_STEEL.id].into()
    }
}

#[async_trait]
impl PumpkinItem for FlintAndSteelItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _block: &Block,
        _server: &Server,
    ) {
        // TODO: check CampfireBlock, CandleBlock and CandleCakeBlock
        let world = player.world().await;
        let pos = location.offset(face.to_offset());
        if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
            let fire_block = FireBlockBase::get_fire_type(&world, &pos).await;

            world
                .set_block_state(&pos, fire_block.default_state_id, BlockFlags::NOTIFY_ALL)
                .await;
            // TODO
        }
    }
}
