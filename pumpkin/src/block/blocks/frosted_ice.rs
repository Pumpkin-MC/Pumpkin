use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{BlockProperties, NetherWartLikeProperties},
    dimension::Dimension,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

use super::ice::melt;
use crate::block::{
    BlockBehaviour, BlockFuture, OnNeighborUpdateArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::world::World;

type FrostedIceProperties = NetherWartLikeProperties;

/// `FrostedIceBlock.MAX_AGE`.
const MAX_AGE: u8 = 3;
/// `FrostedIceBlock.NEIGHBORS_TO_AGE`: fewer than this many frosted-ice neighbors makes the
/// aging/melting scheduled tick run unconditionally (in addition to the flat 1-in-3 chance).
const NEIGHBORS_TO_AGE: usize = 4;
/// `FrostedIceBlock.NEIGHBORS_TO_MELT`: fewer than this many frosted-ice neighbors after a
/// neighbor change instantly melts this block.
const NEIGHBORS_TO_MELT: usize = 2;

/// `net.minecraft.world.level.block.FrostedIceBlock`.
///
/// Extends `IceBlock` in vanilla, so it inherits `IceBlock#randomTick`/`melt` unchanged and adds
/// its own scheduled-tick aging mechanic on top; it does *not* add any player-proximity spreading
/// of its own — that visual (water freezing into frosted ice in a radius around the player) is
/// Frost Walker, a separate enchantment that places this block, not a behaviour of the block
/// itself.
#[pumpkin_block("minecraft:frosted_ice")]
pub struct FrostedIceBlock;

impl BlockBehaviour for FrostedIceBlock {
    // Note: `FrostedIceBlock` inherits `IceBlock#randomTick`, but frosted ice's `Blocks.java`
    // registration (`Blocks.FROSTED_ICE`) has no `.randomTicks()` call, unlike plain ice and
    // unlike crimson/warped nylium (both call `.randomTicks()`). A block without that flag never
    // receives random ticks from the server tick loop, so the inherited method is unreachable in
    // vanilla and is deliberately not implemented here. Only the scheduled-tick aging mechanic
    // below is real.

    /// `FrostedIceBlock#onPlace`: schedule the first aging tick 60-120 ticks after placement.
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.schedule_block_tick(
                args.block,
                *args.position,
                rand::rng().random_range(60..=120),
                TickPriority::Normal,
            );
        })
    }

    /// `FrostedIceBlock#tick`: the scheduled-tick aging/melting mechanic.
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if rand::rng().random_range(0..3) == 0
                || fewer_neighbors_than(args.world, args.position, NEIGHBORS_TO_AGE)
            {
                let brightness = if args.world.dimension == Dimension::THE_END {
                    args.world.get_block_light_level(args.position).unwrap_or(0)
                } else {
                    args.world.get_max_local_raw_brightness(args.position)
                };

                let (block, state) = args.world.get_block_and_state(args.position);
                let age = i32::from(FrostedIceProperties::from_state_id(state.id, block).age);
                let threshold = (11 - age - i32::from(state.opacity)).max(0) as u8;

                if brightness > threshold
                    && slightly_melt(args.world, args.position, block, state.id).await
                {
                    for direction in BlockDirection::all() {
                        let neighbor_pos = args.position.offset(direction.to_offset());
                        let (neighbor_block, neighbor_state) =
                            args.world.get_block_and_state(&neighbor_pos);
                        if neighbor_block == &Block::FROSTED_ICE
                            && !slightly_melt(
                                args.world,
                                &neighbor_pos,
                                neighbor_block,
                                neighbor_state.id,
                            )
                            .await
                        {
                            args.world.schedule_block_tick(
                                neighbor_block,
                                neighbor_pos,
                                rand::rng().random_range(20..=40),
                                TickPriority::Normal,
                            );
                        }
                    }
                    return;
                }
            }

            args.world.schedule_block_tick(
                args.block,
                *args.position,
                rand::rng().random_range(20..=40),
                TickPriority::Normal,
            );
        })
    }

    /// `FrostedIceBlock#neighborChanged`: fewer than two frosted-ice neighbors melts instantly.
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.source_block == &Block::FROSTED_ICE
                && fewer_neighbors_than(args.world, args.position, NEIGHBORS_TO_MELT)
            {
                melt(args.world, args.position).await;
            }
        })
    }
}

/// `FrostedIceBlock#slightlyMelt`: bump `AGE` by one, or fully melt once already at `MAX_AGE`.
/// Returns whether the block finished melting (i.e. `AGE` was already maxed out).
async fn slightly_melt(
    world: &Arc<World>,
    position: &BlockPos,
    block: &'static Block,
    state_id: BlockStateId,
) -> bool {
    let mut props = FrostedIceProperties::from_state_id(state_id, block);
    if props.age < MAX_AGE {
        props.age += 1;
        // `level.setBlock(pos, state, 2)`: listener notification only, no neighbor update -
        // the aging loop below drives neighbor scheduling explicitly instead.
        world
            .set_block_state(
                position,
                props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
        false
    } else {
        melt(world, position).await;
        true
    }
}

/// `FrostedIceBlock`'s neighbor-counting method (vanilla misspells the method name): counts
/// orthogonal frosted-ice neighbors, short circuiting as soon as `limit` is reached.
fn fewer_neighbors_than(world: &World, position: &BlockPos, limit: usize) -> bool {
    let mut count = 0;
    for direction in BlockDirection::all() {
        let neighbor_pos = position.offset(direction.to_offset());
        if world.get_block(&neighbor_pos) == &Block::FROSTED_ICE {
            count += 1;
            if count >= limit {
                return false;
            }
        }
    }
    true
}
