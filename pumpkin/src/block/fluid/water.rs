use super::flowing_trait::FlowingFluid;
use crate::{
    block::{BlockFuture, BlockMetadata, fluid::FluidBehaviour},
    entity::EntityBase,
    world::World,
};
use pumpkin_data::fluid::Fluid;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority};
use std::sync::Arc;

pub struct FlowingWater;

impl BlockMetadata for FlowingWater {
    fn ids() -> Box<[u16]> {
        [Fluid::FLOWING_WATER.id].into()
    }
}

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
        Box::pin(async move {
            // Avoid rescheduling a fluid tick if one is already queued.
            if !world.is_fluid_tick_scheduled(block_pos, fluid).await {
                world
                    .schedule_fluid_tick(fluid, *block_pos, WATER_FLOW_SPEED, TickPriority::Normal)
                    .await;
            }
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
