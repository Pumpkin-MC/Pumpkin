use super::flowing_trait::FlowingFluid;
use crate::{
    block::{BlockFuture, BlockMetadata, fluid::FluidBehaviour},
    entity::EntityBase,
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection,
    dimension::Dimension,
    fluid::{Falling, Fluid, FluidProperties, Level},
    world::WorldEvent,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};
use std::sync::Arc;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;
use pumpkin_data::damage::DamageType;
use std::sync::atomic::Ordering;

pub struct FlowingLava;

impl BlockMetadata for FlowingLava {
    fn ids() -> Box<[u16]> {
        [Fluid::FLOWING_LAVA.id].into()
    }
}

impl FlowingLava {
    async fn receive_neighbor_fluids(
        &self,
        world: &Arc<World>,
        _fluid: &Fluid,
        block_pos: &BlockPos,
    ) -> bool {
        // Logic to determine if we should replace the fluid with any of (cobble, obsidian, stone, etc.)
        let below_is_soul_soil = world
            .get_block(&block_pos.offset(BlockDirection::Down.to_offset()))
            .await
            == &Block::SOUL_SOIL;
        let old_state_id = world.get_block_state_id(block_pos).await;
        let is_still = old_state_id == Block::LAVA.default_state.id;

        for dir in BlockDirection::all() {
            let neighbor_pos = block_pos.offset(dir.to_offset());
            if world.get_block(&neighbor_pos).await == &Block::WATER {
                let mut new_state_id = if is_still {
                    Block::OBSIDIAN
                } else {
                    Block::COBBLESTONE
                }
                .default_state
                .id;
                if let Some(server) = world.server.upgrade() {
                    let event = crate::plugin::block::block_form::BlockFormEvent::new(
                        world.clone(),
                        old_state_id,
                        new_state_id,
                        *block_pos,
                    );
                    let event = server.plugin_manager.fire(event).await;
                    if event.cancelled {
                        return false;
                    }
                    new_state_id = event.new_state_id;
                }
                world
                    .set_block_state(block_pos, new_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
                world
                    .sync_world_event(WorldEvent::LavaExtinguished, *block_pos, 0)
                    .await;
                return false;
            }
            if below_is_soul_soil && world.get_block(&neighbor_pos).await == &Block::BLUE_ICE {
                let mut new_state_id = Block::BASALT.default_state.id;
                if let Some(server) = world.server.upgrade() {
                    let event = crate::plugin::block::block_form::BlockFormEvent::new(
                        world.clone(),
                        old_state_id,
                        new_state_id,
                        *block_pos,
                    );
                    let event = server.plugin_manager.fire(event).await;
                    if event.cancelled {
                        return false;
                    }
                    new_state_id = event.new_state_id;
                }
                world
                    .set_block_state(block_pos, new_state_id, BlockFlags::NOTIFY_NEIGHBORS)
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

const LAVA_FLOW_SPEED_NETHER: u8 = 10;
const LAVA_FLOW_SPEED_SLOW: u8 = 30;

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
                let flow_speed = self.get_flow_speed(world);
                world
                    .schedule_fluid_tick(fluid, *block_pos, flow_speed, TickPriority::Normal)
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
                let flow_speed = self.get_flow_speed(world);
                world
                    .schedule_fluid_tick(fluid, *block_pos, flow_speed, TickPriority::Normal)
                    .await;
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, entity: &'a dyn EntityBase) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let base_entity = entity.get_entity();
            if !base_entity.entity_type.fire_immune
                && !base_entity.fire_immune.load(Ordering::Relaxed)
            {
                base_entity.set_on_fire_for(15.0);

                // Also apply lava damage
                base_entity.damage(entity, 4.0, DamageType::LAVA).await;
            }
        })
    }
}

impl FlowingFluid for FlowingLava {
    fn get_level_decrease_per_block(&self, world: &World) -> i32 {
        // Ultrawarm logic
        if world.dimension == Dimension::THE_NETHER {
            1
        } else {
            2
        }
    }

    fn get_flow_speed(&self, world: &World) -> u8 {
        // Ultrawarm logic - lava flows faster in the Nether
        if world.dimension == Dimension::THE_NETHER {
            LAVA_FLOW_SPEED_NETHER
        } else {
            LAVA_FLOW_SPEED_SLOW
        }
    }

    fn get_max_flow_distance(&self, world: &World) -> i32 {
        // Ultrawarm logic
        if world.dimension == Dimension::THE_NETHER {
            5
        } else {
            3
        }
    }

    /// Determines if lava can convert to source blocks based on game rules.
    fn can_convert_to_source(&self, world: &Arc<World>) -> bool {
        world.level_info.load().game_rules.lava_source_conversion
    }

    async fn spread_to(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) {
        let new_props = FlowingFluidProperties::from_state_id(state_id, fluid);
        let current_state_id = world.get_block_state_id(pos).await;
        let block = Block::from_state_id(current_state_id);

        if new_props.level == Level::L8 && new_props.falling == Falling::True {
            // Stone creation when lava meets water
            if block == &Block::WATER {
                world
                    .set_block_state(pos, Block::STONE.default_state.id, BlockFlags::NOTIFY_ALL)
                    .await;
                world
                    .sync_world_event(WorldEvent::LavaExtinguished, *pos, 0)
                    .await;
                return;
            }
        }

        // Don't flow into waterlogged blocks
        if block.is_waterlogged(current_state_id) {
            return;
        }

        // Delegate quiescence, replacement and scheduling to the shared helper
        self.apply_spread(world, fluid, pos, state_id, new_props)
            .await;
    }
}
