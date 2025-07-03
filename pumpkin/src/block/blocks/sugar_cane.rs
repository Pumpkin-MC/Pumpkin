use async_trait::async_trait;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, CactusLikeProperties, EnumVariants, Integer0To15},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::pumpkin_block::{
    CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnScheduledTickArgs, PumpkinBlock,
    RandomTickArgs,
};

#[pumpkin_block("minecraft:sugar_cane")]
pub struct SugarCaneBlock;

#[async_trait]
impl PumpkinBlock for SugarCaneBlock {
    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.location).await {
            args.world
                .break_block(args.location, None, BlockFlags::empty())
                .await;
        }
    }

    async fn random_tick(&self, args: RandomTickArgs<'_>) {
        if args
            .world
            .get_block_state(&args.location.up())
            .await
            .is_air()
        {
            let state_id = args.world.get_block_state(args.location).await.id;
            let age = CactusLikeProperties::from_state_id(state_id, args.block).age;
            if age == Integer0To15::L15 {
                args.world
                    .set_block_state(&args.location.up(), state_id, BlockFlags::empty())
                    .await;
                let props = CactusLikeProperties {
                    age: Integer0To15::L0,
                };
                args.world
                    .set_block_state(
                        args.location,
                        props.to_state_id(args.block),
                        BlockFlags::empty(),
                    )
                    .await;
            } else {
                let props = CactusLikeProperties {
                    age: Integer0To15::from_index(age.to_index() + 1),
                };
                args.world
                    .set_block_state(
                        args.location,
                        props.to_state_id(args.block),
                        BlockFlags::empty(),
                    )
                    .await;
            }
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.location).await {
            args.world
                .schedule_block_tick(args.block, *args.location, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.location).await
    }
}

async fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let block_below = block_accessor.get_block(&block_pos.down()).await;

    if block_below == &Block::SUGAR_CANE {
        return true;
    }

    if block_below.is_tagged_with("minecraft:dirt").unwrap()
        || block_below.is_tagged_with("minecraft:sand").unwrap()
    {
        for direction in HorizontalFacing::all() {
            let block = block_accessor
                .get_block(&block_pos.down().offset(direction.to_offset()))
                .await;

            if block == &Block::WATER || block == &Block::FROSTED_ICE {
                return true;
            }
        }
    }

    false
}
