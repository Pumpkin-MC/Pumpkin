use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, BrownMushroomBlockLikeProperties},
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
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};

#[pumpkin_block("minecraft:chorus_plant")]
pub struct ChorusPlantBlock;

impl BlockBehaviour for ChorusPlantBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Compute all 6 face connections immediately so the placed block visually connects to its neighbors.
            get_state_with_connections(args.world, args.block, args.position).await
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_survive(args.block_accessor, args.position).await })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_survive(args.world, args.position).await {
                // Schedule delayed destruction so the whole plant cascades down.
                args.world
                    .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal)
                    .await;
                return args.state_id;
            }

            // Update the single face connection for the direction that changed.
            let neighbor_block = args.world.get_block(args.neighbor_position).await;
            let connect = neighbor_block == &Block::CHORUS_PLANT
                || neighbor_block == &Block::CHORUS_FLOWER
                || (args.direction == BlockDirection::Down
                    && neighbor_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_PLANT));

            let mut props =
                BrownMushroomBlockLikeProperties::from_state_id(args.state_id, args.block);
            match args.direction {
                BlockDirection::Down => props.down = connect,
                BlockDirection::Up => props.up = connect,
                BlockDirection::North => props.north = connect,
                BlockDirection::South => props.south = connect,
                BlockDirection::East => props.east = connect,
                BlockDirection::West => props.west = connect,
            }
            props.to_state_id(args.block)
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Destroy if unsupported; breaking propagates neighbor-update callbacks
            // to connected chorus blocks, which schedule their own ticks.
            if !can_survive(args.world.as_ref(), args.position).await {
                args.world
                    .break_block(args.position, None, BlockFlags::empty())
                    .await;
            }
        })
    }
}

async fn get_state_with_connections(
    block_accessor: &dyn BlockAccessor,
    block: &Block,
    pos: &BlockPos,
) -> BlockStateId {
    let plant_id = Block::CHORUS_PLANT.id;
    let flower_id = Block::CHORUS_FLOWER.id;
    let supports = &tag::Block::MINECRAFT_SUPPORTS_CHORUS_PLANT;

    let connects = |b: &Block| b.id == plant_id || b.id == flower_id;

    let block_down = block_accessor.get_block(&pos.down()).await;
    let block_up = block_accessor.get_block(&pos.up()).await;
    let block_north = block_accessor
        .get_block(&pos.offset(BlockDirection::North.to_offset()))
        .await;
    let block_east = block_accessor
        .get_block(&pos.offset(BlockDirection::East.to_offset()))
        .await;
    let block_south = block_accessor
        .get_block(&pos.offset(BlockDirection::South.to_offset()))
        .await;
    let block_west = block_accessor
        .get_block(&pos.offset(BlockDirection::West.to_offset()))
        .await;

    let props = BrownMushroomBlockLikeProperties {
        down: connects(block_down) || block_down.has_tag(supports),
        up: connects(block_up),
        north: connects(block_north),
        east: connects(block_east),
        south: connects(block_south),
        west: connects(block_west),
    };
    props.to_state_id(block)
}

async fn can_survive(block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let below_pos = pos.down();
    let above_pos = pos.up();

    let (block_below, state_below) = block_accessor.get_block_and_state(&below_pos).await;
    let state_above = block_accessor.get_block_state(&above_pos).await;

    // A horizontal branch is invalid when both the block immediately above and
    // below this one are non-air (enclosed inside something).
    let block_above_or_below = !state_above.is_air() && !state_below.is_air();

    for dir in BlockDirection::horizontal() {
        let neighbor_pos = pos.offset(dir.to_offset());
        let neighbor_block = block_accessor.get_block(&neighbor_pos).await;
        if neighbor_block == &Block::CHORUS_PLANT {
            if block_above_or_below {
                return false;
            }
            let neighbor_below = block_accessor.get_block(&neighbor_pos.down()).await;
            if neighbor_below == &Block::CHORUS_PLANT
                || neighbor_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_PLANT)
            {
                return true;
            }
        }
    }

    block_below == &Block::CHORUS_PLANT
        || block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CHORUS_PLANT)
}
