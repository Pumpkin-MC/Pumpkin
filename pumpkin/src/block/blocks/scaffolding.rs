use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::world::World;
use pumpkin_data::{
    block_properties::{BlockProperties, ScaffoldingLikeProperties},
    Block, BlockDirection,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};
use std::sync::Arc;

#[pumpkin_block("minecraft:scaffolding")]
pub struct ScaffoldingBlock;

impl BlockMetadata for ScaffoldingBlock {
    fn ids() -> Box<[u16]> {
        [Block::SCAFFOLDING.id].into()
    }
}

impl BlockBehaviour for ScaffoldingBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = ScaffoldingLikeProperties::default(args.block);
            props.waterlogged = args.replacing.water_source();

            // Sneaking forces normal placement
            if args.player.sneaking.load(std::sync::atomic::Ordering::Relaxed) {
                return props.to_state_id(args.block);
            }

            // If clicking top of scaffolding → attempt upward placement
            if args.direction == BlockDirection::Up {
                let above = args.position.up();
                let above_block = args.world.get_block(&above).await;

                if above_block == &Block::AIR {
                    // Check height limit (max 7)
                    let height = get_scaffolding_height(args.world, args.position).await;
                    if height < 7 {
                        return props.to_state_id(args.block);
                    }
                }
            }

            // Otherwise place sideways in the clicked direction
            props.to_state_id(args.block)
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            can_survive(args.block_accessor, args.position).await
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                .await;
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                ScaffoldingLikeProperties::from_state_id(args.state_id, args.block);

            let distance = compute_distance(args.world, args.position).await;
            props.distance = distance;

            props.bottom = is_bottom(args.world, args.position).await;

            props.to_state_id(args.block)
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !can_survive(args.world, args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
                return;
            }

            let distance = compute_distance(args.world, args.position).await;
            if distance == 7 {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }
}

async fn can_survive(world: &dyn crate::world::world::BlockAccessor, pos: &BlockPos) -> bool {
    let below = pos.down();
    let below_block = world.get_block(&below).await;

    if below_block == &Block::SCAFFOLDING {
        return true;
    }

    let below_state = world.get_block_state(&below).await;
    below_state.is_full_cube() && below_state.is_solid_block()
}

async fn get_scaffolding_height(world: &World, pos: &BlockPos) -> u8 {
    let mut height = 0;
    let mut current = pos.down();

    while height < 7 {
        let block = world.get_block(&current).await;
        if block != &Block::SCAFFOLDING {
            break;
        }
        height += 1;
        current = current.down();
    }

    height
}

async fn compute_distance(world: &World, pos: &BlockPos) -> u8 {
    // Distance 0 if directly supported
    let below = pos.down();
    let below_block = world.get_block(&below).await;

    if below_block == &Block::SCAFFOLDING {
        let below_state = world.get_block_state_id(&below).await;
        let props = ScaffoldingLikeProperties::from_state_id(below_state, &Block::SCAFFOLDING);
        return props.distance;
    }

    let below_state = world.get_block_state(&below).await;
    if below_state.is_full_cube() && below_state.is_solid_block() {
        return 0;
    }

    // Otherwise search horizontally (max 4 blocks)
    let mut best = 7;

    for dir in BlockDirection::horizontal() {
        let neighbor = pos.offset(dir.to_offset());
        let block = world.get_block(&neighbor).await;

        if block == &Block::SCAFFOLDING {
            let state = world.get_block_state_id(&neighbor).await;
            let props = ScaffoldingLikeProperties::from_state_id(state, &Block::SCAFFOLDING);
            best = best.min(props.distance + 1);
        }
    }

    best
}

async fn is_bottom(world: &World, pos: &BlockPos) -> bool {
    let above = pos.up();
    let above_block = world.get_block(&above).await;
    above_block != &Block::SCAFFOLDING
}
