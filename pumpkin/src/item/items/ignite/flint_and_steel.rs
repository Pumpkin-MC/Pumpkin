use crate::entity::player::Player;
use crate::item::pumpkin_item::ItemMetadata;
use crate::item::pumpkin_item::PumpkinItem;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::item::items::ignite::ignition::Ignition;

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
        item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        server: &Server,
    ) {
        Ignition::ignite_block(
            |world: Arc<World>, pos: BlockPos, new_state_id: u16| async move {
                world
                    .set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            },
            item,
            player,
            location,
            face,
            block,
            server,
        )
        .await;
    }
}
