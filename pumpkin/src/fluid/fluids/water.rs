use async_trait::async_trait;
use pumpkin_data::fluid::{Falling, Fluid, FluidProperties, Level};
use pumpkin_data::block::Block;
use pumpkin_macros::pumpkin_fluid;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;


use crate::{block::registry::BlockActionResult, fluid::pumpkin_fluid::PumpkinFluid, server::Server, world::{BlockFlags, World}};

#[pumpkin_fluid("minecraft:flowing_water")]
pub struct FlowingWater;

const WATER_FLOW_SPEED: u16 = 5;

type FlowingWaterProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

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
        world.schedule_fluid_tick(fluid.id, *block_pos, WATER_FLOW_SPEED).await;
    }

    async fn on_scheduled_tick(&self, world: &World, fluid: &Fluid, block_pos: &BlockPos) {
        log::info!("FlowingWater on_scheduled_tick");
        let block_under = block_pos.down();

        let block = world.get_block(&block_under).await.unwrap();

        if block.id == 0 {
            let mut block_props = FlowingWaterProperties::from_state_id(fluid.id, fluid);
            block_props.level = Level::L2;
            block_props.falling = Falling::False;
            world.set_block_state(&block_under, block_props.to_state_id(&fluid), BlockFlags::NOTIFY_ALL).await;
            let updated_block = world.get_block(&block_under).await.unwrap();
            return;
        }
    }

}
