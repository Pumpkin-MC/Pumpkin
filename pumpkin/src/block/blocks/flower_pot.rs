use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::item::Item;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

pub struct FlowerPotBlock;

impl BlockMetadata for FlowerPotBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:flower_pots").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for FlowerPotBlock {
    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        if !block.eq(&Block::FLOWER_POT) {
            world
                .set_block_state(
                    &location,
                    Block::FLOWER_POT.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let next= match item {
            //FLOWERS
            item if item.eq(&Item::DANDELION) => Block::POTTED_DANDELION.default_state.id,
            item if item.eq(&Item::POPPY) => Block::POTTED_POPPY.default_state.id,
            item if item.eq(&Item::BLUE_ORCHID) => {
                Block::POTTED_BLUE_ORCHID.default_state.id
            }
            item if item.eq(&Item::ALLIUM) => Block::POTTED_ALLIUM.default_state.id,
            item if item.eq(&Item::AZURE_BLUET) => {
                Block::POTTED_AZURE_BLUET.default_state.id
            }
            item if item.eq(&Item::RED_TULIP) => Block::POTTED_RED_TULIP.default_state.id,
            item if item.eq(&Item::ORANGE_TULIP) => {
                Block::POTTED_ORANGE_TULIP.default_state.id
            }
            item if item.eq(&Item::WHITE_TULIP) => {
                Block::POTTED_WHITE_TULIP.default_state.id
            }
            item if item.eq(&Item::PINK_TULIP) => Block::POTTED_PINK_TULIP.default_state.id,
            item if item.eq(&Item::OXEYE_DAISY) => {
                Block::POTTED_OXEYE_DAISY.default_state.id
            }
            item if item.eq(&Item::CORNFLOWER) => Block::POTTED_CORNFLOWER.default_state.id,
            item if item.eq(&Item::LILY_OF_THE_VALLEY) => {
                Block::POTTED_LILY_OF_THE_VALLEY.default_state.id
            }
            item if item.eq(&Item::WITHER_ROSE) => {
                Block::POTTED_WITHER_ROSE.default_state.id
            }
            item if item.eq(&Item::OPEN_EYEBLOSSOM) => {
                Block::POTTED_OPEN_EYEBLOSSOM.default_state.id
            }
            item if item.eq(&Item::CLOSED_EYEBLOSSOM) => {
                Block::POTTED_CLOSED_EYEBLOSSOM.default_state.id
            }

            //SAPLING
            item if item.eq(&Item::OAK_SAPLING) => {
                Block::POTTED_OAK_SAPLING.default_state.id
            }
            item if item.eq(&Item::SPRUCE_SAPLING) => {
                Block::POTTED_SPRUCE_SAPLING.default_state.id
            }
            item if item.eq(&Item::BIRCH_SAPLING) => {
                Block::POTTED_BIRCH_SAPLING.default_state.id
            }
            item if item.eq(&Item::JUNGLE_SAPLING) => {
                Block::POTTED_JUNGLE_SAPLING.default_state.id
            }
            item if item.eq(&Item::ACACIA_SAPLING) => {
                Block::POTTED_ACACIA_SAPLING.default_state.id
            }
            item if item.eq(&Item::DARK_OAK_SAPLING) => {
                Block::POTTED_DARK_OAK_SAPLING.default_state.id
            }
            item if item.eq(&Item::CHERRY_SAPLING) => {
                Block::POTTED_CHERRY_SAPLING.default_state.id
            }
            item if item.eq(&Item::MANGROVE_PROPAGULE) => {
                Block::POTTED_MANGROVE_PROPAGULE.default_state.id
            }
            item if item.eq(&Item::PALE_OAK_SAPLING) => {
                Block::POTTED_PALE_OAK_SAPLING.default_state.id
            }

            //MUSHROOM
            item if item.eq(&Item::RED_MUSHROOM) => {
                Block::POTTED_RED_MUSHROOM.default_state.id
            }
            item if item.eq(&Item::BROWN_MUSHROOM) => {
                Block::POTTED_BROWN_MUSHROOM.default_state.id
            }

            //PLANTS
            item if item.eq(&Item::FERN) => Block::POTTED_FERN.default_state.id,
            item if item.eq(&Item::DEAD_BUSH) => Block::POTTED_DEAD_BUSH.default_state.id,
            item if item.eq(&Item::CACTUS) => Block::POTTED_CACTUS.default_state.id,
            item if item.eq(&Item::BAMBOO) => Block::POTTED_BAMBOO.default_state.id,
            item if item.eq(&Item::AZALEA) => Block::POTTED_AZALEA_BUSH.default_state.id,
            item if item.eq(&Item::FLOWERING_AZALEA) => {
                Block::POTTED_FLOWERING_AZALEA_BUSH.default_state.id
            }

            //NETHER
            item if item.eq(&Item::CRIMSON_FUNGUS) => {
                Block::POTTED_CRIMSON_FUNGUS.default_state.id
            }
            item if item.eq(&Item::WARPED_FUNGUS) => {
                Block::POTTED_WARPED_FUNGUS.default_state.id
            }
            item if item.eq(&Item::CRIMSON_ROOTS) => {
                Block::POTTED_CRIMSON_ROOTS.default_state.id
            }
            item if item.eq(&Item::WARPED_ROOTS) => {
                Block::POTTED_WARPED_ROOTS.default_state.id
            }
            _ => {
                world
                    .set_block_state(
                        &location,
                        Block::FLOWER_POT.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                return BlockActionResult::Consume;
            }
        };
        if block.eq(&Block::FLOWER_POT) {
            world
                .set_block_state(&location, next, BlockFlags::NOTIFY_ALL)
                .await;
        }
        BlockActionResult::Consume
    }

    async fn random_tick(&self, block: &Block, world: &Arc<World>, pos: &BlockPos) {
        if world.dimension_type.eq(&VanillaDimensionType::Overworld)
            || world
                .dimension_type
                .eq(&VanillaDimensionType::OverworldCaves)
        {
            if block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM)
                && world.level_time.blocking_lock().time_of_day > 14500
            {
                world
                    .set_block_state(
                        pos,
                        Block::POTTED_OPEN_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        } else if block.eq(&Block::POTTED_OPEN_EYEBLOSSOM)
            && world.level_time.blocking_lock().time_of_day <= 14500
        {
            world
                .set_block_state(
                    pos,
                    Block::POTTED_CLOSED_EYEBLOSSOM.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }
}
