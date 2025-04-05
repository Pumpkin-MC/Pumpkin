use crate::entity::player::Player;
use crate::item::items::honeycomb::WAXED2UNWAXED;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::BlockFlags;
use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockProperties, PaleOakWoodLikeProperties};
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use std::collections::HashMap;
use std::sync::LazyLock;

pub struct AxeItem;

// Need review. Since I can't verify if a block is equal to another (like mojang did) I'm going to do it with there id.

static STRIPPED_BLOCKS: LazyLock<HashMap<u16, Block>> = LazyLock::new(|| {
    HashMap::from([
        (Block::OAK_WOOD.id, Block::STRIPPED_OAK_WOOD),
        (Block::OAK_LOG.id, Block::STRIPPED_OAK_LOG),
        (Block::DARK_OAK_WOOD.id, Block::STRIPPED_DARK_OAK_WOOD),
        (Block::DARK_OAK_LOG.id, Block::STRIPPED_DARK_OAK_LOG),
        (Block::PALE_OAK_WOOD.id, Block::STRIPPED_PALE_OAK_WOOD),
        (Block::PALE_OAK_LOG.id, Block::STRIPPED_PALE_OAK_LOG),
        (Block::ACACIA_WOOD.id, Block::STRIPPED_ACACIA_WOOD),
        (Block::ACACIA_LOG.id, Block::STRIPPED_ACACIA_LOG),
        (Block::CHERRY_WOOD.id, Block::STRIPPED_CHERRY_WOOD),
        (Block::CHERRY_LOG.id, Block::STRIPPED_CHERRY_LOG),
        (Block::BIRCH_WOOD.id, Block::STRIPPED_BIRCH_WOOD),
        (Block::BIRCH_LOG.id, Block::STRIPPED_BIRCH_LOG),
        (Block::JUNGLE_WOOD.id, Block::STRIPPED_JUNGLE_WOOD),
        (Block::JUNGLE_LOG.id, Block::STRIPPED_JUNGLE_LOG),
        (Block::SPRUCE_WOOD.id, Block::STRIPPED_SPRUCE_WOOD),
        (Block::SPRUCE_LOG.id, Block::STRIPPED_SPRUCE_LOG),
        (Block::WARPED_STEM.id, Block::STRIPPED_WARPED_STEM),
        (Block::WARPED_HYPHAE.id, Block::STRIPPED_WARPED_HYPHAE),
        (Block::CRIMSON_STEM.id, Block::STRIPPED_CRIMSON_STEM),
        (Block::CRIMSON_HYPHAE.id, Block::STRIPPED_CRIMSON_HYPHAE),
        (Block::MANGROVE_WOOD.id, Block::STRIPPED_MANGROVE_WOOD),
        (Block::MANGROVE_LOG.id, Block::STRIPPED_MANGROVE_LOG),
        (Block::BAMBOO_BLOCK.id, Block::STRIPPED_BAMBOO_BLOCK),
    ])
});

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
    #[allow(clippy::too_many_lines)]
    async fn use_on_block(
        &self,
        _item: &Item,
        player: &Player,
        location: BlockPos,
        _face: &BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        let world = player.world().await;

        // First we try to strip the block. by getting his equivalent and applying it the axis.
        let mut replacement_block = STRIPPED_BLOCKS.get(&block.id);
        // If there is a strip equivalent.
        if replacement_block.is_some() {
            // get block state of the old log.
            let log_information = world.get_block_state_id(&location).await.unwrap();
            // get the log properties
            let log_props = PaleOakWoodLikeProperties::from_state_id(log_information, block);
            // create new properties for the new log.
            let mut new_log_properties =
                PaleOakWoodLikeProperties::default(replacement_block.unwrap());
            // Set old axis to the new log.
            new_log_properties.axis = log_props.axis;
            world
                .set_block_state(
                    &location,
                    new_log_properties.to_state_id(replacement_block.unwrap()),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            return;
        }
        // TODO Improve this to conserve block data.
        let block_id = WAXED2UNWAXED.get(&block.id);
        if block_id.is_some() {
            let replacement_block = Block::from_id(*block_id.unwrap());

            if replacement_block.is_some() {
                world
                    .set_block_state(
                        &location,
                        replacement_block.unwrap().default_state_id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                return;
            }
        }
    }
}
