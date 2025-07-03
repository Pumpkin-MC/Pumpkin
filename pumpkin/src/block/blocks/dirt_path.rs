use crate::block::pumpkin_block::CanPlaceAtArgs;
use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::OnScheduledTickArgs;
use crate::block::pumpkin_block::PumpkinBlock;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:dirt_path")]
pub struct DirtPathBlock;

#[async_trait]
impl PumpkinBlock for DirtPathBlock {
    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        // TODO: push up entities
        args.world
            .set_block_state(
                args.location,
                Block::DIRT.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if !can_place_at(args.world, args.location).await {
            return Block::DIRT.default_state.id;
        }

        args.block.default_state.id
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Up && !can_place_at(args.world, args.location).await {
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

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let state = world.get_block_state(&block_pos.up()).await;
    !state.is_solid() // TODO: add fence gate block
}
