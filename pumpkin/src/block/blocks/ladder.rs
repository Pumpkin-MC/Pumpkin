use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};
use crate::world::World;
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

impl BlockBehaviour for LadderBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = LadderLikeProperties::default(args.block);
            props.facing = args.direction.opposite().to_cardinal_direction();
            props.to_state_id(args.block)
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            args.block_accessor
                .get_block_state(&args.use_item_on.unwrap().position)
                .await
                .is_side_solid(args.direction.opposite())
                && args.direction.is_horizontal()
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
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
