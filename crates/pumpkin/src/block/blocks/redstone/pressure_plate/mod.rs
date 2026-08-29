use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};

use crate::{
    block::{OnEntityCollisionArgs, OnStateReplacedArgs},
    world::World,
};

pub mod plate;
pub mod weighted;

#[cfg(test)]
mod tests;

// Vanilla pressure plates detect entities in a centered 14x4x14-pixel volume.
const PRESSURE_PLATE_DETECTION_BOX: BoundingBox = BoundingBox::new_array(
    [1.0 / 16.0, 0.0, 1.0 / 16.0],
    [15.0 / 16.0, 4.0 / 16.0, 15.0 / 16.0],
);

fn detection_box_at(pos: &BlockPos) -> BoundingBox {
    PRESSURE_PLATE_DETECTION_BOX.at_pos(*pos)
}

pub(crate) trait PressurePlate {
    fn on_entity_collision_pp(&self, args: OnEntityCollisionArgs<'_>) {
        let output = self.get_redstone_output(args.block, args.state.id);
        if output == 0 {
            self.update_plate_state(args.world, args.position, args.block, args.state, output);
        }
    }

    fn on_state_replaced_pp(&self, args: OnStateReplacedArgs<'_>) {
        if !args.moved && self.get_redstone_output(args.block, args.old_state_id) > 0 {
            args.world.update_neighbors(args.position, None);
            args.world.update_neighbors(&args.position.down(), None);
        }
    }

    fn update_plate_state(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        state: &BlockState,
        output: u8,
    ) {
        let calc_output = self.calculate_redstone_output(world, block, pos);
        let has_output = calc_output > 0;
        if calc_output != output {
            let next_output = if let Some(server) = world.server.upgrade() {
                let mut event = crate::plugin::block::block_redstone::BlockRedstoneEvent::new(
                    world.clone(),
                    state.id,
                    *pos,
                    i32::from(output),
                    i32::from(calc_output),
                );
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return;
                }
                event.new_current.clamp(0, 15) as u8
            } else {
                calc_output
            };
            let state = self.set_redstone_output(block, state, next_output);
            world.set_block_state(pos, state, BlockFlags::NOTIFY_LISTENERS);
            world.update_neighbors(pos, None);
            world.update_neighbors(&pos.down(), None);
        }
        if has_output {
            world.schedule_block_tick(block, *pos, self.tick_rate(), TickPriority::Normal);
        }
    }

    fn can_pressure_plate_place_at(world: &World, block_pos: &BlockPos) -> bool {
        let floor = world.get_block_state(&block_pos.down());
        floor.is_side_solid(BlockDirection::Up)
    }

    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8;

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId;

    fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8;

    fn tick_rate(&self) -> u8 {
        20
    }
}
