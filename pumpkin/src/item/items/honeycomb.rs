use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
pub struct HoneyCombItem;

impl ItemMetadata for HoneyCombItem {
    fn ids() -> Box<[u16]> {
        [Item::HONEYCOMB.id].into()
    }
}

#[async_trait]
impl PumpkinItem for HoneyCombItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        _face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        // Yes, Minecraft does hardcode these
        let replacement_block = match &block.id {
            id if id == &Block::OXIDIZED_COPPER.id => Block::WAXED_OXIDIZED_COPPER.default_state_id,
            id if id == &Block::WEATHERED_COPPER.id => {
                Block::WAXED_WEATHERED_COPPER.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER.id => Block::WAXED_EXPOSED_COPPER.default_state_id,
            id if id == &Block::COPPER_BLOCK.id => Block::WAXED_COPPER_BLOCK.default_state_id,
            id if id == &Block::OXIDIZED_CHISELED_COPPER.id => {
                Block::WAXED_OXIDIZED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::WEATHERED_CHISELED_COPPER.id => {
                Block::WAXED_WEATHERED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::EXPOSED_CHISELED_COPPER.id => {
                Block::WAXED_EXPOSED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::CHISELED_COPPER.id => Block::WAXED_CHISELED_COPPER.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_GRATE.id => {
                Block::WAXED_OXIDIZED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_GRATE.id => {
                Block::WAXED_WEATHERED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_GRATE.id => {
                Block::WAXED_EXPOSED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::COPPER_GRATE.id => Block::WAXED_COPPER_GRATE.default_state_id,
            id if id == &Block::OXIDIZED_CUT_COPPER.id => {
                Block::WAXED_OXIDIZED_CUT_COPPER.default_state_id
            }
            id if id == &Block::WEATHERED_CUT_COPPER.id => {
                Block::WAXED_WEATHERED_CUT_COPPER.default_state_id
            }
            id if id == &Block::EXPOSED_CUT_COPPER.id => {
                Block::WAXED_EXPOSED_CUT_COPPER.default_state_id
            }
            id if id == &Block::CUT_COPPER.id => Block::WAXED_CUT_COPPER.default_state_id,
            id if id == &Block::OXIDIZED_CUT_COPPER_STAIRS.id => {
                Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::WEATHERED_CUT_COPPER_STAIRS.id => {
                Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::EXPOSED_CUT_COPPER_STAIRS.id => {
                Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::CUT_COPPER_STAIRS.id => {
                Block::WAXED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::OXIDIZED_CUT_COPPER_SLAB.id => {
                Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::WEATHERED_CUT_COPPER_SLAB.id => {
                Block::WAXED_WEATHERED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::EXPOSED_CUT_COPPER_SLAB.id => {
                Block::WAXED_EXPOSED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::CUT_COPPER_SLAB.id => Block::WAXED_CUT_COPPER_SLAB.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_BULB.id => {
                Block::WAXED_OXIDIZED_COPPER_BULB.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_BULB.id => {
                Block::WAXED_WEATHERED_COPPER_BULB.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_BULB.id => {
                Block::WAXED_EXPOSED_COPPER_BULB.default_state_id
            }
            id if id == &Block::COPPER_BULB.id => Block::WAXED_COPPER_BULB.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_DOOR.id => {
                Block::WAXED_OXIDIZED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_DOOR.id => {
                Block::WAXED_WEATHERED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_DOOR.id => {
                Block::WAXED_EXPOSED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::COPPER_DOOR.id => Block::WAXED_COPPER_DOOR.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_TRAPDOOR.id => {
                Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_TRAPDOOR.id => {
                Block::WAXED_WEATHERED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_TRAPDOOR.id => {
                Block::WAXED_EXPOSED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::COPPER_TRAPDOOR.id => Block::WAXED_COPPER_TRAPDOOR.default_state_id,
            _ => block.default_state_id,
        };

        if block.default_state_id != replacement_block {
            let world = player.world().await;

            world
                .set_block_state(&location, replacement_block, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }
}
