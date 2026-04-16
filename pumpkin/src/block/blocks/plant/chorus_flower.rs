use pumpkin_data::{
    Block, BlockDirection,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnScheduledTickArgs,
};

#[pumpkin_block("minecraft:chorus_flower")]
pub struct ChorusFlowerBlock;

impl BlockBehaviour for ChorusFlowerBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_survive(args.block_accessor, args.position).await })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.direction != BlockDirection::Up && !can_survive(args.world, args.position).await
            {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_survive(args.world.as_ref(), args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }
}

async fn can_survive(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let block_below = block_accessor.get_block(&pos.down()).await;

    if block_below == &Block::CHORUS_PLANT
        || block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_FLOWER)
    {
        return true;
    }

    if !block_below.is_air() {
        return false;
    }

    // Below is air: the flower is the tip of a horizontal branch.
    // Exactly one horizontal neighbor must be a chorus plant stem.
    let mut plant_count = 0u32;
    for dir in BlockDirection::horizontal() {
        let neighbor = block_accessor.get_block(&pos.offset(dir.to_offset())).await;
        if neighbor == &Block::CHORUS_PLANT {
            plant_count += 1;
            if plant_count > 1 {
                return false;
            }
        } else if !neighbor.is_air() {
            return false;
        }
    }

    plant_count == 1
}
