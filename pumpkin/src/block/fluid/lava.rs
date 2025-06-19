use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    fluid::{Falling, Fluid, FluidProperties, Level},
    world::WorldEvent,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, world::BlockFlags};

use crate::{block::pumpkin_fluid::PumpkinFluid, entity::EntityBase, world::World};

use super::flowing::FlowingFluid;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

#[pumpkin_block("minecraft:flowing_lava")]
pub struct FlowingLava;

impl FlowingLava {
    async fn receive_neighbor_fluids(
        &self,
        world: &Arc<World>,
        _fluid: &Fluid,
        block_pos: &BlockPos,
    ) -> bool {
        // Logic to determine if we should replace the fluid with any of (cobble, obsidian, stone or basalt)
        let below_is_soul_soil = world
            .get_block(&block_pos.offset(BlockDirection::Down.to_offset()))
            .await
            == Block::SOUL_SOIL;
        let is_still = world.get_block_state_id(block_pos).await == Block::LAVA.default_state_id;

        for dir in BlockDirection::flow_directions() {
            let neighbor_pos = block_pos.offset(dir.opposite().to_offset());
            if world.get_block(&neighbor_pos).await == Block::WATER {
                let block = if is_still {
                    Block::OBSIDIAN
                } else {
                    Block::COBBLESTONE
                };
                world
                    .set_block_state(
                        block_pos,
                        block.default_state_id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                world
                    .sync_world_event(WorldEvent::LavaExtinguished, *block_pos, 0)
                    .await;
                return false;
            }
            if below_is_soul_soil && world.get_block(&neighbor_pos).await == Block::BLUE_ICE {
                world
                    .set_block_state(
                        block_pos,
                        Block::BASALT.default_state_id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                world
                    .sync_world_event(WorldEvent::LavaExtinguished, *block_pos, 0)
                    .await;
                return false;
            }
        }
        true
    }
}

const LAVA_FLOW_SPEED: u16 = 30;

#[async_trait]
impl PumpkinFluid for FlowingLava {
    async fn placed(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id != state_id && self.receive_neighbor_fluids(world, fluid, block_pos).await {
            world
                .schedule_fluid_tick(fluid.id, *block_pos, LAVA_FLOW_SPEED)
                .await;
        }
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, fluid: &Fluid, block_pos: &BlockPos) {
        self.spread_fluid(world, fluid, block_pos).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: &BlockPos,
        _notify: bool,
    ) {
        if self.receive_neighbor_fluids(world, fluid, block_pos).await {
            world
                .schedule_fluid_tick(fluid.id, *block_pos, LAVA_FLOW_SPEED)
                .await;
        }
    }

    async fn on_entity_collision(&self, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        if !base_entity.entity_type.fire_immune {
            base_entity.set_on_fire_for(15.0);
        }
    }
}

#[async_trait]
impl FlowingFluid for FlowingLava {
    //TODO implement ultrawarm logic
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

    async fn spread_to(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) {
        let mut new_props = FlowingFluidProperties::default(fluid);
        new_props.level = Level::L8;
        new_props.falling = Falling::True;
        if state_id == new_props.to_state_id(fluid) {
            // STONE creation
            if world.get_block(pos).await == Block::WATER {
                world
                    .set_block_state(pos, Block::STONE.default_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
                world
                    .sync_world_event(WorldEvent::LavaExtinguished, *pos, 0)
                    .await;
                return;
            }
        }

        if self.is_waterlogged(world, pos).await.is_some() {
            return;
        }

        world
            .set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL)
            .await;
    }
}
