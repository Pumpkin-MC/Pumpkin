//! Vanilla `FallingBlock` (26.2 CFR):
//! - `onPlace` / `updateShape` → `level.scheduleTick(pos, this, getDelayAfterPlace())` (default 2 gt)
//! - `tick` (scheduled, **not** random tick) → if free below, `FallingBlockEntity.fall`
//! - `animateTick` only spawns particles (client); never drives falling

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, GetStateForNeighborUpdateArgs,
        OnNeighborUpdateArgs, OnScheduledTickArgs, PlacedArgs,
    },
    entity::falling::FallingEntity,
};
use pumpkin_data::{
    Block, BlockId, BlockState, BlockStateId,
    tag::{self, Taggable},
};
use pumpkin_world::tick::TickPriority;

pub struct FallingBlock;

/// Vanilla `FallingBlock.getDelayAfterPlace()` — 2 game ticks.
const DELAY_AFTER_PLACE: u8 = 2;

impl FallingBlock {
    /// Vanilla `FallingBlock.isFree`
    #[must_use]
    pub fn can_fall_through(state: &BlockState, block: &Block) -> bool {
        state.is_air()
            || block.has_tag(&tag::Block::MINECRAFT_FIRE)
            || state.is_liquid()
            || state.replaceable()
    }

    /// Schedule a planned tick if one is not already queued (vanilla NTE path).
    fn schedule_fall_tick(world: &crate::world::World, block: &Block, pos: pumpkin_util::math::position::BlockPos) {
        if !world.is_block_tick_scheduled(&pos, block) {
            world.schedule_block_tick(block, pos, DELAY_AFTER_PLACE, TickPriority::Normal);
        }
    }
}

impl BlockMetadata for FallingBlock {
    fn ids() -> Box<[BlockId]> {
        // Vanilla FallingBlock / ColoredFallingBlock / ConcretePowderBlock
        [
            BlockId::GRAVEL,
            BlockId::SAND,
            BlockId::RED_SAND,
            BlockId::WHITE_CONCRETE_POWDER,
            BlockId::ORANGE_CONCRETE_POWDER,
            BlockId::MAGENTA_CONCRETE_POWDER,
            BlockId::LIGHT_BLUE_CONCRETE_POWDER,
            BlockId::YELLOW_CONCRETE_POWDER,
            BlockId::LIME_CONCRETE_POWDER,
            BlockId::PINK_CONCRETE_POWDER,
            BlockId::GRAY_CONCRETE_POWDER,
            BlockId::LIGHT_GRAY_CONCRETE_POWDER,
            BlockId::CYAN_CONCRETE_POWDER,
            BlockId::PURPLE_CONCRETE_POWDER,
            BlockId::BLUE_CONCRETE_POWDER,
            BlockId::BROWN_CONCRETE_POWDER,
            BlockId::GREEN_CONCRETE_POWDER,
            BlockId::RED_CONCRETE_POWDER,
            BlockId::BLACK_CONCRETE_POWDER,
        ]
        .into()
    }
}

impl BlockBehaviour for FallingBlock {
    /// Vanilla `FallingBlock.onPlace` → scheduleTick(delay=2)
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            Self::schedule_fall_tick(args.world, args.block, *args.position);
        })
    }

    /// Vanilla `FallingBlock.updateShape` → scheduleTick (any neighbour change).
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            Self::schedule_fall_tick(args.world, args.block, *args.position);
            args.state_id
        })
    }

    /// Also hook neighborChanged — Pumpkin routes solid-block breaks here as well as
    /// shape updates; vanilla schedules from updateShape only, but both must NTE.
    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            Self::schedule_fall_tick(args.world, args.block, *args.position);
        })
    }

    /// Vanilla `FallingBlock.tick` — **scheduled** tick only (not randomTick).
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Re-validate: block may have changed since schedule.
            let current = args.world.get_block(args.position);
            if current.id != args.block.id {
                return;
            }

            let (below_block, below_state) = args.world.get_block_and_state(&args.position.down());
            if !Self::can_fall_through(below_state, below_block)
                || args.position.0.y < args.world.min_y
            {
                return;
            }

            let state = args.world.get_block_state(args.position);
            // Vanilla FallingBlockEntity.fall: setBlock(fluid legacy, flag 3) + add entity
            FallingEntity::replace_spawn(args.world, *args.position, state.id).await;
        })
    }
}
