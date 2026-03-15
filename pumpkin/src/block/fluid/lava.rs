use super::flowing_trait::FlowingFluid;
use crate::{
    block::blocks::fire::fire::FireBlock,
    block::{BlockFuture, BlockMetadata, fluid::FluidBehaviour},
    entity::EntityBase,
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::blocks_movement,
    damage::DamageType,
    dimension::Dimension,
    fluid::{Falling, Fluid, FluidProperties, Level},
    world::WorldEvent,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};
use std::sync::Arc;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;
use std::sync::atomic::Ordering;

pub struct FlowingLava;

impl BlockMetadata for FlowingLava {
    fn ids() -> Box<[u16]> {
        [Fluid::FLOWING_LAVA.id].into()
    }
}

impl FlowingLava {
    fn can_spread_fire_around(world: &Arc<World>, pos: &BlockPos) -> bool {
        let spread_radius = world
            .level_info
            .load()
            .game_rules
            .fire_spread_radius_around_player;

        if spread_radius == 0 {
            return false;
        }
        if spread_radius == -1 {
            return true;
        }

        world
            .get_closest_player(pos.to_centered_f64(), spread_radius as f64)
            .is_some()
    }

    fn is_flammable_state(block_state: &BlockState) -> bool {
        let block = Block::from_state_id(block_state.id);
        if block.is_waterlogged(block_state.id) {
            return false;
        }
        block
            .flammable
            .as_ref()
            .is_some_and(|flammable| flammable.burn_chance > 0)
    }

    fn is_flammable(world: &Arc<World>, pos: &BlockPos) -> bool {
        world
            .get_block_state_if_loaded(pos)
            .is_some_and(Self::is_flammable_state)
    }

    fn has_flammable_neighbours(world: &Arc<World>, pos: &BlockPos) -> bool {
        BlockDirection::all()
            .iter()
            .any(|dir| Self::is_flammable(world, &pos.offset(dir.to_offset())))
    }

    fn can_resolve_fire_state_without_loading(world: &Arc<World>, pos: &BlockPos) -> bool {
        if !world.is_loaded(pos) || !world.is_loaded(&pos.down()) {
            return false;
        }

        BlockDirection::all()
            .iter()
            .all(|dir| world.is_loaded(&pos.offset(dir.to_offset())))
    }

    async fn ignite_fire_if_possible(world: &Arc<World>, pos: &BlockPos) {
        if !Self::can_resolve_fire_state_without_loading(world, pos) {
            return;
        }

        let fire_state_id = FireBlock
            .get_state_for_position(world.as_ref(), &Block::FIRE, pos)
            .await;
        world
            .set_block_state(pos, fire_state_id, BlockFlags::NOTIFY_ALL)
            .await;
    }

    async fn receive_neighbor_fluids(
        world: &Arc<World>,
        _fluid: &Fluid,
        block_pos: &BlockPos,
    ) -> bool {
        // Logic to determine if we should replace the fluid with any of (cobble, obsidian, stone, etc.)
        let below_is_soul_soil = world
            .get_block(&block_pos.offset(BlockDirection::Down.to_offset()))
            .await
            == &Block::SOUL_SOIL;
        let is_still = world.get_block_state_id(block_pos).await == Block::LAVA.default_state.id;

        for dir in BlockDirection::all() {
            let neighbor_pos = block_pos.offset(dir.to_offset());
            if world.get_block(&neighbor_pos).await == &Block::WATER {
                if dir == BlockDirection::Down {
                    return true;
                }
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
                && Self::receive_neighbor_fluids(world, fluid, block_pos).await
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
            if Self::receive_neighbor_fluids(world, fluid, block_pos).await {
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

    fn random_tick<'a>(
        &'a self,
        _fluid: &'a Fluid,
        world: &'a Arc<World>,
        block_pos: &'a BlockPos,
    ) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !Self::can_spread_fire_around(world, block_pos) {
                return;
            }

            let passes = rand::random_range(0..3);
            if passes > 0 {
                let mut test_pos = *block_pos;

                for _ in 0..passes {
                    test_pos = test_pos.offset(Vector3::new(
                        rand::random_range(-1..=1),
                        1,
                        rand::random_range(-1..=1),
                    ));

                    if !world.is_loaded(&test_pos) {
                        return;
                    }

                    let Some(block_state) = world.get_block_state_if_loaded(&test_pos) else {
                        return;
                    };

                    if block_state.is_air() {
                        if Self::has_flammable_neighbours(world, &test_pos) {
                            Self::ignite_fire_if_possible(world, &test_pos).await;
                            return;
                        }
                    } else if blocks_movement(&block_state, Block::from_state_id(block_state.id).id)
                    {
                        return;
                    }
                }
            } else {
                for _ in 0..3 {
                    let test_pos = block_pos.offset(Vector3::new(
                        rand::random_range(-1..=1),
                        0,
                        rand::random_range(-1..=1),
                    ));

                    if !world.is_loaded(&test_pos) {
                        return;
                    }

                    let above_pos = test_pos.up();
                    if !world.is_loaded(&above_pos) {
                        return;
                    }

                    if world
                        .get_block_state_if_loaded(&above_pos)
                        .is_some_and(BlockState::is_air)
                        && Self::is_flammable(world, &test_pos)
                    {
                        Self::ignite_fire_if_possible(world, &above_pos).await;
                    }
                }
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
