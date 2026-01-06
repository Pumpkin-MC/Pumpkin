use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, CocoaLikeProperties, Integer0To2},
    item::Item,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
use rand::Rng;

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, RandomTickArgs, UseWithItemArgs, registry::BlockActionResult,
};

#[pumpkin_block("minecraft:cocoa")]
pub struct CocoaBlock;

impl BlockBehaviour for CocoaBlock {
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item_lock = args.item_stack.lock().await;
            if item_lock.get_item() != &Item::BONE_MEAL {
                return BlockActionResult::Pass;
            }

            let state_id = args.world.get_block_state_id(args.position).await;

            let mut props = CocoaLikeProperties::from_state_id(state_id, &Block::COCOA);

            props.age = match props.age {
                Integer0To2::L0 => Integer0To2::L1,
                Integer0To2::L1 => Integer0To2::L2,
                Integer0To2::L2 => return BlockActionResult::Pass,
            };

            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(&Block::COCOA),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;

            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            log::info!(
                "direction = {:?} - horizontal_facing = {:?}",
                args.direction,
                args.direction.to_horizontal_facing()
            );
            if !can_place_at(args.world, args.direction, args.position).await {
                return Block::AIR.default_state.id;
            }

            let mut props = CocoaLikeProperties::default(&Block::COCOA);
            props.facing = match args.direction.to_horizontal_facing() {
                Some(v) => v,
                None => return Block::AIR.default_state.id,
            };

            props.age = Integer0To2::L0;

            props.to_state_id(&Block::COCOA)
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(
            async move { can_place_at(args.block_accessor, args.direction, args.position).await },
        )
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !is_valid_placement(args.world, args.position).await {
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state) = args.world.get_block_and_state(args.position).await;
            if block != &Block::COCOA {
                return;
            }
            let props = CocoaLikeProperties::from_state_id(state.id, &Block::COCOA);
            let offset_block = args
                .world
                .get_block(&args.position.offset(props.facing.to_offset()))
                .await;

            if !offset_block.has_tag(&tag::Block::MINECRAFT_JUNGLE_LOGS) {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if rand::rng().random_range(0..5) == 0 {
                let state_id = args.world.get_block_state_id(args.position).await;

                let mut props = CocoaLikeProperties::from_state_id(state_id, &Block::COCOA);

                props.age = match props.age {
                    Integer0To2::L0 => Integer0To2::L1,
                    Integer0To2::L1 => Integer0To2::L2,
                    Integer0To2::L2 => return,
                };

                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(&Block::COCOA),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}

/// Checks if a cocoa bean could be placed at the location with a given direction
pub async fn can_place_at(
    block_accessor: &dyn BlockAccessor,
    block_direction: BlockDirection,
    position: &BlockPos,
) -> bool {
    let Some(facing) = block_direction.to_horizontal_facing() else {
        return false;
    };
    let block = block_accessor
        .get_block(&position.offset(facing.to_offset()))
        .await;

    block.has_tag(&tag::Block::MINECRAFT_JUNGLE_LOGS)
}

/// Checks if a placed cocoa bean is valid
pub async fn is_valid_placement(block_accessor: &impl BlockAccessor, position: &BlockPos) -> bool {
    let (block, state) = block_accessor.get_block_and_state(position).await;
    if block != &Block::COCOA {
        return false;
    }
    let props = CocoaLikeProperties::from_state_id(state.id, &Block::COCOA);
    let offset_block = block_accessor
        .get_block(&position.offset(props.facing.to_offset()))
        .await;

    offset_block.has_tag(&tag::Block::MINECRAFT_JUNGLE_LOGS)
}
