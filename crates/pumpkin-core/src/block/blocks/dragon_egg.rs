use crate::block::blocks::falling::FallingBlock;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BrokenArgs, NormalUseArgs, OnScheduledTickArgs, PathComputationType, PlacedArgs,
};
use crate::world::World;
use pumpkin_data::BlockState;
use pumpkin_data::world::WorldEvent;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use rand::{RngExt, rng};
use std::sync::Arc;

#[pumpkin_block("minecraft:dragon_egg")]
pub struct DragonEggBlock;

impl DragonEggBlock {
    const TELEPORT_RADIUS_XZ: i32 = 16;
    const TELEPORT_RADIUS_Y: i32 = 8;

    fn teleport(world: &Arc<World>, pos: &BlockPos) {
        for _ in 0..1000 {
            let test_pos = pos.add(
                rng().random_range(0..Self::TELEPORT_RADIUS_XZ)
                    - rng().random_range(0..Self::TELEPORT_RADIUS_XZ),
                rng().random_range(0..Self::TELEPORT_RADIUS_Y)
                    - rng().random_range(0..Self::TELEPORT_RADIUS_Y),
                rng().random_range(0..Self::TELEPORT_RADIUS_XZ)
                    - rng().random_range(0..Self::TELEPORT_RADIUS_XZ),
            );

            let state = world.get_block_state(&test_pos);
            let below_state = world.get_block_state(&test_pos.down());

            if state.is_air() && !below_state.is_air() {
                let packed_diff = pos.pack_difference_in_position(
                    &test_pos,
                    Self::TELEPORT_RADIUS_XZ,
                    Self::TELEPORT_RADIUS_Y,
                    Self::TELEPORT_RADIUS_XZ,
                );
                world.sync_world_event(WorldEvent::ParticlesDragonEggTeleport, *pos, packed_diff);
                let current_state = world.get_block_state(pos);
                world.set_block_state(
                    &test_pos,
                    current_state.id,
                    pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                );
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

impl BlockBehaviour for DragonEggBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .schedule_block_tick(args.block, *args.position, 5, TickPriority::Normal);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::teleport(args.world, args.position);
        BlockActionResult::Success
    }

    // TODO this should be attack but this does not exist yet in pumpkin
    // it is hardcoded for left clicking note blocks at JavaClient.handle_player_action
    fn broken(&self, args: BrokenArgs<'_>) {
        Self::teleport(args.world, args.position);
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        FallingBlock::on_scheduled_tick(&FallingBlock, args);
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
