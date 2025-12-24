use std::sync::Arc;

use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority};

use crate::{
    block::{BlockFuture, fluid::FluidBehaviour},
    entity::EntityBase,
    world::World,
};

use super::flowing::FlowingFluid;

#[pumpkin_block("minecraft:flowing_water")]
pub struct FlowingWater;

const WATER_FLOW_SPEED: u8 = 5;

impl FluidBehaviour for FlowingWater {
    fn placed<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        state_id: BlockStateId,
        block_pos: &'a BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if old_state_id != state_id {
                world
                    .schedule_fluid_tick(fluid, *block_pos, WATER_FLOW_SPEED, TickPriority::Normal)
                    .await;
            }
        })
    }

    fn on_scheduled_tick<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {
            self.on_scheduled_tick_internal(world, fluid, block_pos)
                .await;
        })
    }

    fn on_neighbor_update<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
        _notify: bool,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async {
            world
                .schedule_fluid_tick(fluid, *block_pos, WATER_FLOW_SPEED, TickPriority::Normal)
                .await;
        })
    }

    fn on_entity_collision<'a>(&'a self, entity: &'a dyn EntityBase) -> BlockFuture<'a, ()> {
        Box::pin(async {
            entity.get_entity().extinguish();
        })
    }
}

impl FlowingFluid for FlowingWater {
    fn get_level_decrease_per_block(&self, _world: &World) -> i32 {
        1
    }

    fn get_max_flow_distance(&self, _world: &World) -> i32 {
        4
    }

    fn can_convert_to_source(&self, _world: &Arc<World>) -> bool {
        //TODO add game rule check for water conversion
        true
    }
}
