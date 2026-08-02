use std::sync::Arc;

use pumpkin_data::{Block, dimension::Dimension};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, RandomTickArgs};
use crate::world::World;

#[pumpkin_block("minecraft:ice")]
pub struct IceBlock;

impl BlockBehaviour for IceBlock {
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // `IceBlock.randomTick` melts once the block light exceeds
            // `11 - state.getLightBlock()`. Java's `getLightBlock()` is `BlockState::opacity`
            // here, and ice has an opacity of 1, so the effective threshold is a block light
            // level above 10 (one lower than the flat `> 11` used for snow layers).
            let opacity = args.world.get_block_state(args.position).opacity;
            if args.world.get_block_light_level(args.position).unwrap_or(0)
                <= 11u8.saturating_sub(opacity)
            {
                return;
            }

            melt(args.world, args.position).await;
        })
    }
}

/// `IceBlock#melt`: ultrawarm dimensions evaporate the ice, everything else leaves a water
/// source behind. Melting never drops an ice item, so the state is overwritten instead of
/// broken. Shared with `FrostedIceBlock`, which inherits this method unchanged.
pub(super) async fn melt(world: &Arc<World>, position: &BlockPos) {
    if world.dimension == Dimension::THE_NETHER {
        world
            .set_block_state(
                position,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        return;
    }

    world
        .set_block_state(
            position,
            Block::WATER.default_state.id,
            BlockFlags::NOTIFY_ALL,
        )
        .await;
    // `level.neighborChanged(pos, Blocks.WATER, null)`: tell the fresh water source about its
    // own surroundings so that it starts flowing right away.
    world.update_neighbor(position, &Block::WATER).await;
}
