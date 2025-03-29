use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::fluid::{Falling, Fluid, FluidProperties, Level};
use pumpkin_macros::pumpkin_fluid;
use pumpkin_util::math::position::BlockPos;

use crate::{
    fluid::pumpkin_fluid::PumpkinFluid,
    world::World,
};

use super::flowing_fluid::FlowingFluid;

#[pumpkin_fluid("minecraft:flowing_water")]
pub struct FlowingWater;

const WATER_FLOW_SPEED: u16 = 5;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

#[async_trait]
impl PumpkinFluid for FlowingWater {
    async fn placed(
        &self,
        world: &World,
        fluid: &Fluid,
        _state_id: u16,
        block_pos: &BlockPos,
        _old_state_id: u16,
        _notify: bool,
    ) {
        world
            .schedule_fluid_tick(fluid.id, *block_pos, WATER_FLOW_SPEED)
            .await;
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, fluid: &Fluid, block_pos: &BlockPos) {
        self.spread_fluid(world, fluid, block_pos).await;
    }
}

#[async_trait]
impl FlowingFluid for FlowingWater {
    async fn get_drop_off(&self) -> i32 {
        1
    }

    async fn get_source(&self, fluid: &Fluid, falling: bool) -> FlowingFluidProperties {
        let mut source_props = FlowingFluidProperties::default(fluid);
        source_props.level = Level::L8;
        source_props.falling = if falling { Falling::True } else { Falling::False };
        source_props
    }

    async fn get_flowing(&self, fluid: &Fluid, level: Level, falling: bool) -> FlowingFluidProperties {
        let mut flowing_props = FlowingFluidProperties::default(fluid);
        flowing_props.level = level;
        flowing_props.falling = if falling { Falling::True } else { Falling::False };
        flowing_props
    }

    async fn get_slope_find_distance(&self) -> i32 {
        4
    }

    async fn can_convert_to_source(&self, _world: &Arc<World>) -> bool {
        //TODO add game rule check for water conversion
        true
    }

    fn is_same_fluid(&self, fluid: &Fluid, other_state_id: u16) -> bool {
        fluid.id == other_state_id
    }
}
