use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::fire::FireBlockBase;
use crate::entity::player::Player;
use crate::item::items::ignition::ignition::Ignition;
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
        Ignition::ignite_block(
            async |world, pos, block| {
                let state_block = match block {
                    Some(block) => block,
                    None => return,
                };

                world
                    .set_block_state(&pos, state_block.default_state_id, BlockFlags::NOTIFY_ALL)
                    .await;

                // TODO
            },
            _item,
            player,
            location,
            face,
            _block,
            _server,
        )
        .await;
    }
}
