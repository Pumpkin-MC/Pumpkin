use crate::block::blocks::falling::FallingBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, GetStateForNeighborUpdateArgs, NormalUseArgs, OnScheduledTickArgs,
    PathComputationType, PlacedArgs,
};
use crate::world::World;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use rand::{RngExt, rng};
use std::sync::Arc;

/// Vanilla `DragonEggBlock` delay (in ticks) before checking and initiating falling physics.
const FALL_DELAY_TICKS: u8 = 5;

/// Maximum number of attempts to find a random valid air block to teleport to.
const MAX_TELEPORT_ATTEMPTS: u32 = 1000;

/// Horizontal teleport search radius (±16 blocks).
const TELEPORT_HORIZONTAL_RANGE: std::ops::Range<i32> = -16..16;

/// Vertical teleport search radius (±8 blocks).
const TELEPORT_VERTICAL_RANGE: std::ops::Range<i32> = -8..8;

#[pumpkin_block("minecraft:dragon_egg")]
pub struct DragonEggBlock;

impl DragonEggBlock {
    fn teleport(world: &Arc<World>, pos: &BlockPos) {
        for _ in 0..MAX_TELEPORT_ATTEMPTS {
            let x = pos.0.x + rng().random_range(TELEPORT_HORIZONTAL_RANGE);
            let y = pos.0.y + rng().random_range(TELEPORT_VERTICAL_RANGE);
            let z = pos.0.z + rng().random_range(TELEPORT_HORIZONTAL_RANGE);
            let test_pos = BlockPos::new(x, y, z);

            let state = world.get_block_state(&test_pos);

            if state.is_air() {
                let current_state = world.get_block_state(pos);
                if world
                    .set_block_state_if(
                        &test_pos,
                        current_state.id,
                        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                        |_| true,
                    )
                    .is_some()
                {
                    world.set_block_state(
                        pos,
                        pumpkin_data::Block::AIR.default_state.id,
                        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                    );
                    return;
                }
            }
        }
    }
}

impl BlockBehaviour for DragonEggBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world.schedule_block_tick(
            args.block,
            *args.position,
            FALL_DELAY_TICKS,
            TickPriority::Normal,
        );
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::teleport(args.world, args.position);
        BlockActionResult::Success
    }

    // Dragon egg is typically teleported when attacked
    fn broken(&self, args: BrokenArgs<'_>) {
        Self::teleport(args.world, args.position);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.world.schedule_block_tick(
            args.block,
            *args.position,
            FALL_DELAY_TICKS,
            TickPriority::Normal,
        );
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        FallingBlock::on_scheduled_tick(&FallingBlock, args);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
