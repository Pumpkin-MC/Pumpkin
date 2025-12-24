use std::sync::Arc;

use pumpkin_data::{
    Block, BlockDirection,
    fluid::{Falling, Fluid, FluidProperties, Level},
    world::WorldEvent,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};

use crate::{
    block::{
        BlockFuture,
        fluid::{FluidBehaviour, flowing::FluidFuture},
    },
    entity::EntityBase,
    world::World,
};

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
            == &Block::SOUL_SOIL;
        let is_still = world.get_block_state_id(block_pos).await == Block::LAVA.default_state.id;

        for dir in BlockDirection::flow_directions() {
            let neighbor_pos = block_pos.offset(dir.opposite().to_offset());
            if world.get_block(&neighbor_pos).await == &Block::WATER {
                let block = if is_still {
                    Block::OBSIDIAN
                } else {
                    Block::COBBLESTONE
                };
                world
                    .set_block_state(
                        block_pos,
                        block.default_state.id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                world
                    .sync_world_event(WorldEvent::LavaExtinguished, *block_pos, 0)
                    .await;
                return false;
            }
            if below_is_soul_soil && world.get_block(&neighbor_pos).await == &Block::BLUE_ICE {
                world
                    .set_block_state(
                        block_pos,
                        Block::BASALT.default_state.id,
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

const LAVA_FLOW_SPEED: u8 = 30;

impl FluidBehaviour for FlowingLava {
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
            if old_state_id != state_id
                && self.receive_neighbor_fluids(world, fluid, block_pos).await
            {
                world
                    .schedule_fluid_tick(fluid, *block_pos, LAVA_FLOW_SPEED, TickPriority::Normal)
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
        Box::pin(async move {
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
            if self.receive_neighbor_fluids(world, fluid, block_pos).await {
                world
                    .schedule_fluid_tick(fluid, *block_pos, LAVA_FLOW_SPEED, TickPriority::Normal)
                    .await;
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, entity: &'a dyn EntityBase) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let base_entity = entity.get_entity();
            if !base_entity.entity_type.fire_immune {
                base_entity.set_on_fire_for(15.0);
            }
        })
    }
}

impl FlowingFluid for FlowingLava {
    fn get_level_decrease_per_block(&self, world: &World) -> i32 {
        // ultrawarm logic
        if world.dimension_type == VanillaDimensionType::TheNether {
            1
        } else {
            2
        }
    }

    fn get_max_flow_distance(&self, world: &World) -> i32 {
        // ultrawarm logic
        if world.dimension_type == VanillaDimensionType::TheNether {
            4
        } else {
            2
        }
    }

    fn can_convert_to_source(&self, _world: &Arc<World>) -> bool {
        //TODO add game rule check for lava conversion
        false
    }

    fn spread_to<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        pos: &'a BlockPos,
        state_id: BlockStateId,
    ) -> FluidFuture<'a, ()> {
        Box::pin(async move {
            let mut new_props = FlowingFluidProperties::default(fluid);
            new_props.level = Level::L8;
            new_props.falling = Falling::True;
            if state_id == new_props.to_state_id(fluid) {
                // STONE creation
                if world.get_block(pos).await == &Block::WATER {
                    world
                        .set_block_state(pos, Block::STONE.default_state.id, BlockFlags::NOTIFY_ALL)
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
        })
    }
}
