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
        face: &BlockDirection,
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
            _ => block.default_state_id,
        };

        // Yes, Minecraft does hardcode these
        if replacement_block != 0 {
            let world = player.world().await;

            world
                .set_block_state(&location, replacement_block, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }
}
