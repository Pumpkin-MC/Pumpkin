use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};

use crate::{
    block::{OnEntityCollisionArgs, OnScheduledTickArgs, OnStateReplacedArgs},
    world::World,
};

pub mod plate;
pub mod weighted;

pub(crate) trait PressurePlate {
    async fn on_entity_collision_pp(&self, args: OnEntityCollisionArgs<'_>) {
        let output = self.get_redstone_output(args.block, args.state.id);
        if output == 0 {
            self.update_plate_state(args.world, args.position, args.block, args.state, output)
                .await;
        }
    }

    async fn on_scheduled_tick_pp(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position).await;
        let output = self.get_redstone_output(args.block, state.id);
        if output > 0 {
            self.update_plate_state(args.world, args.position, args.block, state, output)
                .await;
        }
    }

    async fn on_state_replaced_pp(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved && self.get_redstone_output(args.block, args.old_state_id) > 0 {
            args.world.update_neighbors(args.position, None).await;
            args.world
                .update_neighbors(&args.position.down(), None)
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
            let mut next_output = calc_output;
            if let Some(server) = world.server.upgrade() {
                let event_block = Block::from_id(block.id);
                let event = crate::plugin::block::block_redstone::BlockRedstoneEvent::new(
                    event_block,
                    *pos,
                    world.uuid,
                    i32::from(output),
                    i32::from(calc_output),
                );
                let event = server.plugin_manager.fire(event).await;
                if event.cancelled {
                    return;
                }
                next_output = event.new_current.clamp(0, 15) as u8;
            }
            let state = self.set_redstone_output(block, state, next_output);
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

    async fn can_pressure_plate_place_at(world: &World, block_pos: &BlockPos) -> bool {
        let floor = world.get_block_state(&block_pos.down()).await;
        floor.is_side_solid(BlockDirection::Up)
    }

    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8;

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId;

    async fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8;

    fn tick_rate(&self) -> u8 {
        20
    }
}
