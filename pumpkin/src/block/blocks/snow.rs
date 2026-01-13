use pumpkin_data::block_properties::BlockProperties as _;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{Integer1To8, SnowLikeProperties},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::block::{
    BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs,
    UseWithItemArgs, registry::BlockActionResult,
};

#[pumpkin_block("minecraft:snow")]
pub struct LayeredSnowBlock;

impl BlockBehaviour for LayeredSnowBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at(args.world, args.position).await {
                return Block::AIR.default_state.id;
            }
            let mut props = SnowLikeProperties::default(args.block);
            props.layers = Integer1To8::L1;
            props.to_state_id(&Block::SNOW)
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item = {
                let lock = args.item_stack.lock().await;
                lock.item
            };

            if item == &Item::SNOW {
                let pos = if args.hit.face.is_horizontal() {
                    &args.position.offset(args.hit.face.to_offset())
                } else {
                    args.position
                };
                if !can_place_at(args.world.as_ref(), pos).await {
                    return BlockActionResult::Pass;
                }
                let (block, state_id) = args.world.get_block_and_state_id(pos).await;

                if block != &Block::SNOW {
                    return BlockActionResult::Pass;
                }

                let mut props = SnowLikeProperties::from_state_id(state_id, &Block::SNOW);
                props.layers = match props.layers {
                    Integer1To8::L1 => Integer1To8::L2,
                    Integer1To8::L2 => Integer1To8::L3,
                    Integer1To8::L3 => Integer1To8::L4,
                    Integer1To8::L4 => Integer1To8::L5,
                    Integer1To8::L5 => Integer1To8::L6,
                    Integer1To8::L6 => Integer1To8::L7,
                    Integer1To8::L7 => Integer1To8::L8,
                    Integer1To8::L8 => {
                        args.world
                            .set_block_state(
                                pos,
                                Block::SNOW_BLOCK.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        return BlockActionResult::Success;
                    }
                };

                let state_id = props.to_state_id(&Block::SNOW);
                args.world
                    .set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                return BlockActionResult::Success;
            }
            BlockActionResult::Pass
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_place_at(args.world.as_ref(), args.position).await {
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
            if !can_place_at(args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            args.state_id
        })
    }
}

async fn can_place_at(block_accessor: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let state = block_accessor.get_block_state(&position.down()).await;
    state.is_side_solid(BlockDirection::Up)
}
