use std::sync::Arc;

use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, chunk::TickPriority, world::BlockFlags};

use crate::{
    block::pumpkin_block::{OnEntityCollisionArgs, OnScheduledTickArgs, OnStateReplacedArgs},
    world::World,
};

pub mod plate;
pub mod weighted;

pub(crate) trait PressurePlate {
    async fn on_entity_collision_pp(&self, args: OnEntityCollisionArgs<'_>) {
        let output = self.get_redstone_output(args.block, args.state.id);
        if output == 0 {
            self.update_plate_state(args.world, args.location, args.block, args.state, output)
                .await;
        }
    }

    async fn on_scheduled_tick_pp(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        let output = self.get_redstone_output(args.block, state.id);
        if output > 0 {
            self.update_plate_state(args.world, args.location, args.block, state, output)
                .await;
        }
    }

    async fn on_state_replaced_pp(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved && self.get_redstone_output(args.block, args.old_state_id) > 0 {
            args.world.update_neighbors(args.location, None).await;
            args.world
                .update_neighbors(&args.location.down(), None)
                .await;
        }
    }

    async fn update_plate_state(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        state: &BlockState,
        output: u8,
    ) {
        let calc_output = self.calculate_redstone_output(world, block, pos).await;
        let has_output = calc_output > 0;
        if calc_output != output {
            let state = self.set_redstone_output(block, state, calc_output);
            world
                .set_block_state(pos, state, BlockFlags::NOTIFY_LISTENERS)
                .await;
            world.update_neighbors(pos, None).await;
            world.update_neighbors(&pos.down(), None).await;
        }
        if has_output {
            world
                .schedule_block_tick(block, *pos, self.tick_rate(), TickPriority::Normal)
                .await;
        }
    }

    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8;

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId;

    async fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8;

    fn tick_rate(&self) -> u16 {
        20
    }
}
