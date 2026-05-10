use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, Facing, LadderLikeProperties};
use pumpkin_data::{Block, BlockDirection, FacingExt, HorizontalFacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;

#[pumpkin_block("minecraft:ladder")]
pub struct LadderBlock;

impl BlockBehaviour for LadderBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let clicked_pos = args.use_item_on.position;
            let (clicked_block, clicked_block_state_id) =
                args.world.get_block_and_state_id(&clicked_pos).await;
            if clicked_block == &Block::LADDER {
                //you can't click on a ladder and place a ladder
                let props =
                    LadderLikeProperties::from_state_id(clicked_block_state_id, clicked_block);
                let sub = args.position.0.sub(&clicked_pos.0);
                let dir = horizontal_facing_from_offset(sub).unwrap();
                if props.facing == dir.to_horizontal_facing().unwrap() {
                    return Block::AIR.default_state.id;
                }
            }
            let mut props = LadderLikeProperties::default(args.block);

            let directions = args.player.get_entity().get_entity_facing_order();
            for dir in directions {
                if dir == Facing::Up || dir == Facing::Down {
                    continue;
                }
                if !can_place_ladder_at(args.world, args.position, dir.to_block_direction()).await {
                    continue;
                }
                props.facing = dir
                    .opposite()
                    .to_block_direction()
                    .to_horizontal_facing()
                    .unwrap();
                return props.to_state_id(args.block);
            }
            Block::AIR.default_state.id
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            for dir in BlockDirection::horizontal() {
                let Some(world) = args.world else {
                    //this won't happen
                    return false;
                };
                if can_place_ladder_at(world, args.position, dir).await {
                    return true;
                }
            }
            false
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = LadderLikeProperties::from_state_id(args.state_id, args.block);
            if props.facing.to_block_direction().opposite() == args.direction
                && !can_place_ladder_at(
                    args.world,
                    args.position,
                    props.facing.to_block_direction().opposite(),
                )
                .await
            {
                return 0;
            }
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position).await;
            if Block::from_state_id(state_id) != &Block::LADDER {
                return;
            }
            let props = LadderLikeProperties::from_state_id(state_id, args.block);
            if !can_place_ladder_at(
                args.world,
                args.position,
                props.facing.to_block_direction().opposite(),
            )
            .await
            {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
        })
    }
}
#[must_use]
pub const fn horizontal_facing_from_offset(offset: Vector3<i32>) -> Option<BlockDirection> {
    match (offset.x, offset.y, offset.z) {
        (0, 0, -1) => Some(BlockDirection::North),
        (0, 0, 1) => Some(BlockDirection::South),
        (-1, 0, 0) => Some(BlockDirection::West),
        (1, 0, 0) => Some(BlockDirection::East),
        _ => None,
    }
}
async fn can_place_ladder_at(world: &World, block_pos: &BlockPos, facing: BlockDirection) -> bool {
    world
        .get_block_state(&block_pos.offset(facing.to_offset()))
        .await
        .is_side_solid(facing.opposite())
}
