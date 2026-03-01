use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use pumpkin_world::{BlockStateId, world::BlockAccessor};

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnScheduledTickArgs,
};

#[pumpkin_macros::pumpkin_block("minecraft:cactus_flower")]
pub struct CactusFlowerBlock;

impl BlockBehaviour for CactusFlowerBlock {
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !self
                .can_place_at_internal(args.world.as_ref(), args.position)
                .await
            {
                // Destroy the block with drops enabled
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Instead of immediately becoming AIR (which skips drops), schedule a tick
            // Only schedule if the neighbor update is NOT from UP (similar to ChorusFlowerBlock)
            if args.direction != BlockDirection::Up
                && !self.can_place_at_internal(args.world, args.position).await
            {
                // Schedule a tick for the next game tick
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }

            args.state_id
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            self.can_place_at_internal(args.block_accessor, args.position)
                .await
        })
    }
}

impl CactusFlowerBlock {
    async fn can_place_at_internal(&self, world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        let block_below = world.get_block(&block_pos.down()).await;
        // Cactus flower can be placed on cactus
        block_below == &Block::CACTUS
    }
}
