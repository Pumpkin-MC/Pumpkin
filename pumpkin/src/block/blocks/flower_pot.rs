use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

#[pumpkin_block("minecraft:flower_pot")]
pub struct FlowerPotBlock;

#[async_trait]
impl PumpkinBlock for FlowerPotBlock {
    async fn use_with_item(
        &self,
        _block: &Block,
        _player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let next;
        match item {
            //FLOWERS
            item if item.eq(&Item::DANDELION) => next = Block::POTTED_DANDELION.default_state_id,
            item if item.eq(&Item::POPPY) => next = Block::POTTED_POPPY.default_state_id,
            item if item.eq(&Item::BLUE_ORCHID) => {
                next = Block::POTTED_BLUE_ORCHID.default_state_id
            }
            item if item.eq(&Item::ALLIUM) => next = Block::POTTED_ALLIUM.default_state_id,
            item if item.eq(&Item::AZURE_BLUET) => {
                next = Block::POTTED_AZURE_BLUET.default_state_id
            }
            item if item.eq(&Item::RED_TULIP) => next = Block::POTTED_RED_TULIP.default_state_id,
            item if item.eq(&Item::ORANGE_TULIP) => {
                next = Block::POTTED_ORANGE_TULIP.default_state_id
            }
            item if item.eq(&Item::WHITE_TULIP) => {
                next = Block::POTTED_WHITE_TULIP.default_state_id
            }
            item if item.eq(&Item::PINK_TULIP) => next = Block::POTTED_PINK_TULIP.default_state_id,
            item if item.eq(&Item::OXEYE_DAISY) => {
                next = Block::POTTED_OXEYE_DAISY.default_state_id
            }
            item if item.eq(&Item::CORNFLOWER) => next = Block::POTTED_CORNFLOWER.default_state_id,
            item if item.eq(&Item::LILY_OF_THE_VALLEY) => {
                next = Block::POTTED_LILY_OF_THE_VALLEY.default_state_id
            }
            item if item.eq(&Item::WITHER_ROSE) => {
                next = Block::POTTED_WITHER_ROSE.default_state_id
            }
            item if item.eq(&Item::OPEN_EYEBLOSSOM) => {
                next = Block::POTTED_OPEN_EYEBLOSSOM.default_state_id
            }
            item if item.eq(&Item::CLOSED_EYEBLOSSOM) => {
                next = Block::POTTED_CLOSED_EYEBLOSSOM.default_state_id
            }

            //SAPLING
            item if item.eq(&Item::OAK_SAPLING) => {
                next = Block::POTTED_OAK_SAPLING.default_state_id
            }
            item if item.eq(&Item::SPRUCE_SAPLING) => {
                next = Block::POTTED_SPRUCE_SAPLING.default_state_id
            }
            item if item.eq(&Item::BIRCH_SAPLING) => {
                next = Block::POTTED_BIRCH_SAPLING.default_state_id
            }
            item if item.eq(&Item::JUNGLE_SAPLING) => {
                next = Block::POTTED_JUNGLE_SAPLING.default_state_id
            }
            item if item.eq(&Item::ACACIA_SAPLING) => {
                next = Block::POTTED_ACACIA_SAPLING.default_state_id
            }
            item if item.eq(&Item::DARK_OAK_SAPLING) => {
                next = Block::POTTED_DARK_OAK_SAPLING.default_state_id
            }
            item if item.eq(&Item::CHERRY_SAPLING) => {
                next = Block::POTTED_CHERRY_SAPLING.default_state_id
            }
            item if item.eq(&Item::MANGROVE_PROPAGULE) => {
                next = Block::POTTED_MANGROVE_PROPAGULE.default_state_id
            }
            item if item.eq(&Item::PALE_OAK_SAPLING) => {
                next = Block::POTTED_PALE_OAK_SAPLING.default_state_id
            }

            //MUSHROOM
            item if item.eq(&Item::RED_MUSHROOM) => {
                next = Block::POTTED_RED_MUSHROOM.default_state_id
            }
            item if item.eq(&Item::BROWN_MUSHROOM) => {
                next = Block::POTTED_BROWN_MUSHROOM.default_state_id
            }

            //PLANTS
            item if item.eq(&Item::FERN) => next = Block::POTTED_FERN.default_state_id,
            item if item.eq(&Item::DEAD_BUSH) => next = Block::POTTED_DEAD_BUSH.default_state_id,
            item if item.eq(&Item::CACTUS) => next = Block::POTTED_CACTUS.default_state_id,
            item if item.eq(&Item::BAMBOO) => next = Block::POTTED_BAMBOO.default_state_id,
            item if item.eq(&Item::AZALEA) => next = Block::POTTED_AZALEA_BUSH.default_state_id,
            item if item.eq(&Item::FLOWERING_AZALEA) => {
                next = Block::POTTED_FLOWERING_AZALEA_BUSH.default_state_id
            }

            //NETHER
            item if item.eq(&Item::CRIMSON_FUNGUS) => {
                next = Block::POTTED_CRIMSON_FUNGUS.default_state_id
            }
            item if item.eq(&Item::WARPED_FUNGUS) => {
                next = Block::POTTED_WARPED_FUNGUS.default_state_id
            }
            item if item.eq(&Item::CRIMSON_ROOTS) => {
                next = Block::POTTED_CRIMSON_ROOTS.default_state_id
            }
            item if item.eq(&Item::WARPED_ROOTS) => {
                next = Block::POTTED_WARPED_ROOTS.default_state_id
            }
            _ => return BlockActionResult::Continue,
        }
        world
            .set_block_state(&location, next, BlockFlags::NOTIFY_ALL)
            .await;
        BlockActionResult::Consume
    }
}
