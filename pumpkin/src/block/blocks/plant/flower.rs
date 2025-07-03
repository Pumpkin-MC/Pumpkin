use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::{RegistryKey, Tagable, get_tag_values};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_world::world::BlockFlags;

use crate::block::pumpkin_block::{BlockMetadata, CanPlaceAtArgs, PumpkinBlock, RandomTickArgs};

pub struct FlowerBlock;

impl BlockMetadata for FlowerBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:flowers/small").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for FlowerBlock {
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let block_below = args.block_accessor.get_block(&args.location.down()).await;
        block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == &Block::FARMLAND
    }

    async fn random_tick(&self, args: RandomTickArgs<'_>) {
        //TODO add trail particule
        if args
            .world
            .dimension_type
            .eq(&VanillaDimensionType::Overworld)
            || args
                .world
                .dimension_type
                .eq(&VanillaDimensionType::OverworldCaves)
        {
            if args.block.eq(&Block::CLOSED_EYEBLOSSOM)
                && args.world.level_time.lock().await.time_of_day > 14500
            {
                args.world
                    .set_block_state(
                        args.location,
                        Block::OPEN_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            } else if args.block.eq(&Block::OPEN_EYEBLOSSOM)
                && args.world.level_time.lock().await.time_of_day <= 14500
            {
                args.world
                    .set_block_state(
                        args.location,
                        Block::CLOSED_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
    }
}
