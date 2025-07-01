use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::flower_pot_transformations::get_potted_item;
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

    #[allow(clippy::too_many_arguments)]
    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        //Place the flower inside the pot
        if block.eq(&Block::FLOWER_POT) {
            if let Some(potted_block_id) = get_potted_item(item.id) {
                world
                    .set_block_state(
                        &location,
                        Block::from_id(potted_block_id).unwrap().default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
            return BlockActionResult::Consume;
        }

        //if the player have an item that can be potted in his hand, nothing happens
        if let Some(_potted_block_id) = get_potted_item(item.id) {
            return BlockActionResult::Consume;
        }

        //get the flower + empty the pot
        world
            .set_block_state(
                &location,
                Block::FLOWER_POT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        BlockActionResult::Consume
    }

    async fn random_tick(&self, block: &Block, world: &Arc<World>, pos: &BlockPos) {
        if world.dimension_type.eq(&VanillaDimensionType::Overworld)
            || world
                .dimension_type
                .eq(&VanillaDimensionType::OverworldCaves)
        {
            if block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM)
                && world.level_time.lock().await.time_of_day > 14500
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
            && world.level_time.lock().await.time_of_day <= 14500
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
