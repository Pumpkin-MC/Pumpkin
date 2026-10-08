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
use rand::{RngExt, rng};
use std::sync::Arc;

#[pumpkin_block("minecraft:dragon_egg")]
pub struct DragonEggBlock;

impl DragonEggBlock {
    // DragonEggBlock.getDelayAfterPlace
    const DELAY_AFTER_PLACE: u8 = 5;

    fn teleport(world: &Arc<World>, pos: &BlockPos) {
        for _ in 0..1000 {
            let x = pos.0.x + rng().random_range(-16..16);
            let y = pos.0.y + rng().random_range(-8..8);
            let z = pos.0.z + rng().random_range(-16..16);
            let test_pos = BlockPos::new(x, y, z);

            let state = world.get_block_state(&test_pos);
            let below_state = world.get_block_state(&test_pos.down());

            if state.is_air() && !below_state.is_air() {
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
        FallingBlock::placed_with_delay(&args, Self::DELAY_AFTER_PLACE);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        FallingBlock::get_state_for_neighbor_update_with_delay(&args, Self::DELAY_AFTER_PLACE)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        Self::teleport(args.world, args.position);
        BlockActionResult::Success
    }

    // Dragon egg is typically teleported when attacked
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
