use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::fluid::Fluid;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::{block::pumpkin_fluid::PumpkinFluid, entity::EntityBase, world::World};

use super::flowing::FlowingFluid;

#[pumpkin_block("minecraft:flowing_lava")]
pub struct FlowingLava;

const LAVA_FLOW_SPEED: u16 = 30;

#[async_trait]
impl PumpkinFluid for FlowingLava {
    async fn placed(
        &self,
        world: &World,
        fluid: &Fluid,
        _state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        world
            .schedule_fluid_tick(fluid.id, *block_pos, LAVA_FLOW_SPEED)
            .await;
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, fluid: &Fluid, block_pos: &BlockPos) {
        self.spread_fluid(world, fluid, block_pos).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &World,
        fluid: &Fluid,
        block_pos: &BlockPos,
        _notify: bool,
    ) {
        world
            .schedule_fluid_tick(fluid.id, *block_pos, LAVA_FLOW_SPEED)
            .await;
    }

    async fn on_entity_collision(&self, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        if !base_entity.entity_type.fire_immune {
            base_entity.set_on_fire_for(15.0);
        }

        if let Some(living) = entity.get_living_entity() {
            let halved_fall = living.fall_distance.load() / 2.0;
            if halved_fall != 0.0 {
                living.fall_distance.store(halved_fall);
            }
        }
    }
}

#[async_trait]
impl FlowingFluid for FlowingLava {
    async fn get_drop_off(&self) -> i32 {
        2
    }

    async fn get_slope_find_distance(&self) -> i32 {
        2
    }

    async fn can_convert_to_source(&self, _world: &Arc<World>) -> bool {
        //TODO add game rule check for lava conversion
        false
    }
}
