use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        Item::get_tag_values("#minecraft:axes")
            .expect("This is a valid vanilla tag")
            .iter()
            .map(|key| {
                Item::from_registry_key(key)
                    .expect("We just got this key from the registry")
                    .id
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

#[async_trait]
impl PumpkinItem for AxeItem {
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        _face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        let replacement_block = match &block.id {
            id if id == &Block::OAK_LOG.id => Block::STRIPPED_OAK_LOG.default_state_id,
            id if id == &Block::BIRCH_LOG.id => Block::STRIPPED_BIRCH_LOG.default_state_id,
            id if id == &Block::SPRUCE_LOG.id => Block::STRIPPED_SPRUCE_LOG.default_state_id,
            id if id == &Block::ACACIA_LOG.id => Block::STRIPPED_ACACIA_LOG.default_state_id,
            id if id == &Block::CHERRY_LOG.id => Block::STRIPPED_CHERRY_LOG.default_state_id,
            id if id == &Block::JUNGLE_LOG.id => Block::STRIPPED_JUNGLE_LOG.default_state_id,
            id if id == &Block::PALE_OAK_LOG.id => Block::STRIPPED_PALE_OAK_LOG.default_state_id,
            id if id == &Block::DARK_OAK_LOG.id => Block::STRIPPED_DARK_OAK_LOG.default_state_id,
            id if id == &Block::MANGROVE_LOG.id => Block::STRIPPED_MANGROVE_LOG.default_state_id,
            id if id == &Block::OAK_WOOD.id => Block::STRIPPED_OAK_WOOD.default_state_id,
            id if id == &Block::BIRCH_WOOD.id => Block::STRIPPED_BIRCH_WOOD.default_state_id,
            id if id == &Block::SPRUCE_WOOD.id => Block::STRIPPED_SPRUCE_WOOD.default_state_id,
            id if id == &Block::ACACIA_WOOD.id => Block::STRIPPED_ACACIA_WOOD.default_state_id,
            id if id == &Block::CHERRY_WOOD.id => Block::STRIPPED_CHERRY_WOOD.default_state_id,
            id if id == &Block::JUNGLE_WOOD.id => Block::STRIPPED_JUNGLE_WOOD.default_state_id,
            id if id == &Block::PALE_OAK_WOOD.id => Block::STRIPPED_PALE_OAK_WOOD.default_state_id,
            id if id == &Block::DARK_OAK_WOOD.id => Block::STRIPPED_DARK_OAK_WOOD.default_state_id,
            id if id == &Block::MANGROVE_WOOD.id => Block::STRIPPED_MANGROVE_WOOD.default_state_id,
            id if id == &Block::OXIDIZED_COPPER.id => Block::WEATHERED_COPPER.default_state_id,
            id if id == &Block::WEATHERED_COPPER.id => Block::EXPOSED_COPPER.default_state_id,
            id if id == &Block::EXPOSED_COPPER.id => Block::COPPER_BLOCK.default_state_id,
            id if id == &Block::WAXED_OXIDIZED_COPPER.id => Block::OXIDIZED_COPPER.default_state_id,
            id if id == &Block::WAXED_WEATHERED_COPPER.id => {
                Block::WEATHERED_COPPER.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_COPPER.id => Block::EXPOSED_COPPER.default_state_id,
            id if id == &Block::WAXED_COPPER_BLOCK.id => Block::COPPER_BLOCK.default_state_id,
            id if id == &Block::OXIDIZED_CHISELED_COPPER.id => {
                Block::WEATHERED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::WEATHERED_CHISELED_COPPER.id => {
                Block::EXPOSED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::EXPOSED_CHISELED_COPPER.id => {
                Block::CHISELED_COPPER.default_state_id
            }
            id if id == &Block::WAXED_OXIDIZED_CHISELED_COPPER.id => {
                Block::OXIDIZED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_CHISELED_COPPER.id => {
                Block::WEATHERED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_CHISELED_COPPER.id => {
                Block::EXPOSED_CHISELED_COPPER.default_state_id
            }
            id if id == &Block::WAXED_CHISELED_COPPER.id => Block::CHISELED_COPPER.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_GRATE.id => {
                Block::WEATHERED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_GRATE.id => {
                Block::EXPOSED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_GRATE.id => Block::COPPER_GRATE.default_state_id,
            id if id == &Block::WAXED_OXIDIZED_COPPER_GRATE.id => {
                Block::OXIDIZED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_COPPER_GRATE.id => {
                Block::WEATHERED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_COPPER_GRATE.id => {
                Block::EXPOSED_COPPER_GRATE.default_state_id
            }
            id if id == &Block::WAXED_COPPER_GRATE.id => Block::COPPER_GRATE.default_state_id,
            id if id == &Block::OXIDIZED_CUT_COPPER.id => {
                Block::WEATHERED_CUT_COPPER.default_state_id
            }
            id if id == &Block::WEATHERED_CUT_COPPER.id => {
                Block::EXPOSED_CUT_COPPER.default_state_id
            }
            id if id == &Block::EXPOSED_CUT_COPPER.id => Block::CUT_COPPER.default_state_id,
            id if id == &Block::WAXED_OXIDIZED_CUT_COPPER.id => {
                Block::OXIDIZED_CUT_COPPER.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_CUT_COPPER.id => {
                Block::WEATHERED_CUT_COPPER.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_CUT_COPPER.id => {
                Block::EXPOSED_CUT_COPPER.default_state_id
            }
            id if id == &Block::WAXED_CUT_COPPER.id => Block::CUT_COPPER.default_state_id,
            id if id == &Block::OXIDIZED_CUT_COPPER_STAIRS.id => {
                Block::WEATHERED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::WEATHERED_CUT_COPPER_STAIRS.id => {
                Block::EXPOSED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::EXPOSED_CUT_COPPER_STAIRS.id => {
                Block::CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.id => {
                Block::OXIDIZED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.id => {
                Block::WEATHERED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.id => {
                Block::EXPOSED_CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::WAXED_CUT_COPPER_STAIRS.id => {
                Block::CUT_COPPER_STAIRS.default_state_id
            }
            id if id == &Block::OXIDIZED_CUT_COPPER_SLAB.id => {
                Block::WEATHERED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::WEATHERED_CUT_COPPER_SLAB.id => {
                Block::EXPOSED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::EXPOSED_CUT_COPPER_SLAB.id => {
                Block::CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.id => {
                Block::OXIDIZED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_CUT_COPPER_SLAB.id => {
                Block::WEATHERED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_CUT_COPPER_SLAB.id => {
                Block::EXPOSED_CUT_COPPER_SLAB.default_state_id
            }
            id if id == &Block::WAXED_CUT_COPPER_SLAB.id => Block::CUT_COPPER_SLAB.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_BULB.id => {
                Block::WEATHERED_COPPER_BULB.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_BULB.id => {
                Block::EXPOSED_COPPER_BULB.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_BULB.id => Block::COPPER_BULB.default_state_id,
            id if id == &Block::WAXED_OXIDIZED_COPPER_BULB.id => {
                Block::OXIDIZED_COPPER_BULB.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_COPPER_BULB.id => {
                Block::WEATHERED_COPPER_BULB.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_COPPER_BULB.id => {
                Block::EXPOSED_COPPER_BULB.default_state_id
            }
            id if id == &Block::WAXED_COPPER_BULB.id => Block::COPPER_BULB.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_DOOR.id => {
                Block::WEATHERED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_DOOR.id => {
                Block::EXPOSED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_DOOR.id => Block::COPPER_DOOR.default_state_id,
            id if id == &Block::WAXED_OXIDIZED_COPPER_DOOR.id => {
                Block::OXIDIZED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_COPPER_DOOR.id => {
                Block::WEATHERED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_COPPER_DOOR.id => {
                Block::EXPOSED_COPPER_DOOR.default_state_id
            }
            id if id == &Block::WAXED_COPPER_DOOR.id => Block::COPPER_DOOR.default_state_id,
            id if id == &Block::OXIDIZED_COPPER_TRAPDOOR.id => {
                Block::WEATHERED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::WEATHERED_COPPER_TRAPDOOR.id => {
                Block::EXPOSED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::EXPOSED_COPPER_TRAPDOOR.id => {
                Block::COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.id => {
                Block::OXIDIZED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::WAXED_WEATHERED_COPPER_TRAPDOOR.id => {
                Block::WEATHERED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::WAXED_EXPOSED_COPPER_TRAPDOOR.id => {
                Block::EXPOSED_COPPER_TRAPDOOR.default_state_id
            }
            id if id == &Block::WAXED_COPPER_TRAPDOOR.id => Block::COPPER_TRAPDOOR.default_state_id,

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
