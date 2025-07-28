use crate::block::pumpkin_block::{
    CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PumpkinBlock,
};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::{BlockProperties, LadderLikeProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block("minecraft:ladder")]
pub struct LadderBlock;

#[async_trait]
impl PumpkinBlock for LadderBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = LadderLikeProperties::default(args.block);
        props.facing = args.direction.opposite().to_cardinal_direction();
        props.to_state_id(args.block)
    }
    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.block_accessor
            .get_block_state(&args.use_item_on.unwrap().position)
            .await
            .is_side_solid(args.direction.opposite())
            && args.direction.is_horizontal()
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position).await {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                .await;
        }
        args.state_id
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world, args.position).await {
            args.world
                .break_block(args.position, None, BlockFlags::empty())
                .await;
        }
    }
}

async fn can_place_at(world: &World, position: &BlockPos) -> bool {
    let props = LadderLikeProperties::from_state_id(
        world.get_block_state_id(position).await,
        world.get_block(position).await,
    );
    let pos;
    let direction;
    match props.r#facing {
        HorizontalFacing::North => {
            pos = position.add(0, 0, 1);
            direction = BlockDirection::North;
        }
        HorizontalFacing::South => {
            pos = position.add(0, 0, -1);
            direction = BlockDirection::South;
        }
        HorizontalFacing::West => {
            pos = position.add(1, 0, 0);
            direction = BlockDirection::West;
        }
        HorizontalFacing::East => {
            pos = position.add(-1, 0, 0);
            direction = BlockDirection::East;
        }
    }

    world
        .get_block_state(&pos)
        .await
        .is_side_solid(direction.opposite())
        && direction.is_horizontal()
}
