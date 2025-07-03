use crate::block::pumpkin_block::{
    BlockMetadata, NormalUseArgs, PumpkinBlock, RandomTickArgs, UseWithItemArgs,
};
use crate::block::registry::BlockActionResult;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::flower_pot_transformations::get_potted_item;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_world::world::BlockFlags;

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
    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        if !args.block.eq(&Block::FLOWER_POT) {
            args.world
                .set_block_state(
                    args.location,
                    Block::FLOWER_POT.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let item = args.item_stack.lock().await.item;
        //Place the flower inside the pot
        if args.block.eq(&Block::FLOWER_POT) {
            if let Some(potted_block_id) = get_potted_item(item.id) {
                args.world
                    .set_block_state(
                        args.location,
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
        args.world
            .set_block_state(
                args.location,
                Block::FLOWER_POT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        BlockActionResult::Consume
    }

    async fn random_tick(&self, args: RandomTickArgs<'_>) {
        if args
            .world
            .dimension_type
            .eq(&VanillaDimensionType::Overworld)
            || args
                .world
                .dimension_type
                .eq(&VanillaDimensionType::OverworldCaves)
        {
            if args.block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM)
                && args.world.level_time.lock().await.time_of_day > 14500
            {
                args.world
                    .set_block_state(
                        args.location,
                        Block::POTTED_OPEN_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        } else if args.block.eq(&Block::POTTED_OPEN_EYEBLOSSOM)
            && args.world.level_time.lock().await.time_of_day <= 14500
        {
            args.world
                .set_block_state(
                    args.location,
                    Block::POTTED_CLOSED_EYEBLOSSOM.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }
}
