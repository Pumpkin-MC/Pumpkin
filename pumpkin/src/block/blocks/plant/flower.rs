use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::pumpkin_block::{
    BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, PumpkinBlock,
};

use crate::block::pumpkin_block::RandomTickArgs;

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
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
        .await
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
                && args.world.level_time.lock().await.time_of_day % 24000 > 14500
            {
                args.world
                    .set_block_state(
                        args.position,
                        Block::OPEN_EYEBLOSSOM.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        }
        if args.block.eq(&Block::OPEN_EYEBLOSSOM)
            && args.world.level_time.lock().await.time_of_day % 24000 <= 14500
        {
            args.world
                .set_block_state(
                    args.position,
                    Block::CLOSED_EYEBLOSSOM.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }
}

impl PlantBlockBase for FlowerBlock {}
