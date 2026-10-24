use super::flowing_trait::FlowingFluid;
use crate::{
    block::{FluidMetadata, fluid::FluidBehaviour},
    entity::EntityBase,
    world::World,
};
use pumpkin_data::BlockStateId;
use pumpkin_data::fluid::Fluid;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use std::sync::Arc;

pub struct FlowingWater;

impl FluidMetadata for FlowingWater {
    fn ids() -> Box<[u16]> {
        [Fluid::FLOWING_WATER.id].into()
    }
}

const WATER_FLOW_SPEED: u8 = 5;

impl FluidBehaviour for FlowingWater {
    fn placed(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id != state_id {
            world.schedule_fluid_tick(fluid, *block_pos, WATER_FLOW_SPEED, TickPriority::Normal);
        }
    }

    fn on_scheduled_tick(&self, world: &Arc<World>, _fluid: &Fluid, block_pos: &BlockPos) {
        Self.on_scheduled_tick_internal(world, &Fluid::FLOWING_WATER, block_pos);
    }

    fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: &BlockPos,
        _notify: bool,
    ) {
        // Avoid rescheduling a fluid tick if one is already queued.
        if !world.is_fluid_tick_scheduled(block_pos, fluid) {
            world.schedule_fluid_tick(fluid, *block_pos, WATER_FLOW_SPEED, TickPriority::Normal);
        }
    }

    fn on_entity_collision(&self, entity: &dyn EntityBase) {
        entity.get_entity().extinguish();
    }
}

impl FlowingFluid for FlowingWater {
    fn get_level_decrease_per_block(&self, _world: &World) -> i32 {
        1
    }

    fn get_flow_speed(&self, _world: &World) -> u8 {
        WATER_FLOW_SPEED
    }

    fn get_max_flow_distance(&self, _world: &World) -> i32 {
        4
    }

    /// Determines if water can convert to source blocks based on game rules.
    fn can_convert_to_source(&self, world: &Arc<World>) -> bool {
        world.level_info.load().game_rules.water_source_conversion
    }
}
