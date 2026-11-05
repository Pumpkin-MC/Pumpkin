use super::flowing_trait::FlowingFluid;
use crate::{
    block::{
        FluidMetadata,
        blocks::fire::{fire::FireBlock, soul_fire::SoulFireBlock},
        fluid::FluidBehaviour,
    },
    entity::EntityBase,
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection, BlockState, BlockStateId,
    damage::DamageType,
    fluid::{Falling, Fluid, FluidProperties, Level},
    tag::{self, Taggable},
    world::WorldEvent,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use std::sync::Arc;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

pub struct FlowingLava;

impl FluidMetadata for FlowingLava {
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

        if spread_radius == -1 {
            return true;
        }
        if spread_radius <= 0 {
            return false;
        }

        let target = Vector3::new(f64::from(pos.0.x), f64::from(pos.0.y), f64::from(pos.0.z));
        let max_distance_sq = (spread_radius as f64).powi(2);
        world.players.load().iter().any(|player| {
            !player.is_spectator()
                && player
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&target)
                    < max_distance_sq
        })
    }

    fn is_flammable(world: &Arc<World>, pos: &BlockPos) -> bool {
        world
            .get_block_state_if_loaded(pos)
            .is_some_and(BlockState::burnable)
    }

    fn has_flammable_neighbours(world: &Arc<World>, pos: &BlockPos) -> bool {
        BlockDirection::all()
            .iter()
            .any(|dir| Self::is_flammable(world, &pos.offset(dir.to_offset())))
    }

    fn can_resolve_fire_state_without_loading(world: &Arc<World>, pos: &BlockPos) -> bool {
        let (x, z) = (pos.0.x, pos.0.z);
        let loaded = |x: i32, z: i32| world.level.is_chunk_loaded(&Vector2::new(x >> 4, z >> 4));

        loaded(x, z)
            && (x & 15 != 0 || loaded(x - 1, z))
            && (x & 15 != 15 || loaded(x + 1, z))
            && (z & 15 != 0 || loaded(x, z - 1))
            && (z & 15 != 15 || loaded(x, z + 1))
    }

    fn ignite_fire_if_possible(world: &Arc<World>, fire_pos: &BlockPos, state_pos: &BlockPos) {
        if !Self::can_resolve_fire_state_without_loading(world, state_pos) {
            return;
        }

        let fire_state_id = if SoulFireBlock::is_soul_base(world.get_block(&state_pos.down())) {
            Block::SOUL_FIRE.default_state.id
        } else {
            FireBlock.get_state_for_position(world.as_ref(), &Block::FIRE, state_pos)
        };
        world.set_block_state(fire_pos, fire_state_id, BlockFlags::NOTIFY_ALL);
    }

    fn receive_neighbor_fluids(world: &Arc<World>, block_pos: &BlockPos) -> bool {
        // Logic to determine if we should replace the fluid with any of (cobble, obsidian, stone, etc.)
        let below_is_soul_soil = world
            .get_block(&block_pos.offset(BlockDirection::Down.to_offset()))
            == &Block::SOUL_SOIL;
        let is_still = world.get_block_state_id(block_pos) == Block::LAVA.default_state.id;

        for dir in BlockDirection::all() {
            // Vanilla tests every direction but Down here. Lava landing in water
            // below is turned to stone by `spread_to`, not by this scan, so Down
            // is skipped rather than ending it - a lava block with water beneath
            // *and* water beside it still forms cobblestone.
            if dir == BlockDirection::Down {
                continue;
            }

            let neighbor_pos = block_pos.offset(dir.to_offset());
            if world
                .get_fluid(&neighbor_pos)
                .has_tag(&pumpkin_data::tag::Fluid::MINECRAFT_WATER)
            {
                let block = if is_still {
                    Block::OBSIDIAN
                } else {
                    Block::COBBLESTONE
                };
                world.set_block_state(block_pos, block.default_state.id, BlockFlags::NOTIFY_ALL);
                world.sync_world_event(WorldEvent::LavaFizz, *block_pos, 0);
                return false;
            }
            if below_is_soul_soil && world.get_block(&neighbor_pos) == &Block::BLUE_ICE {
                if let Some(server) = world.server.upgrade() {
                    let mut event =
                        crate::plugin::api::events::block::block_form::BlockFormEvent::new(
                            *block_pos,
                            &Block::BASALT,
                        );
                    server.plugin_manager.fire_blocking(&server, &mut event);
                    if event.cancelled {
                        return false;
                    }
                }
                world.set_block_state(
                    block_pos,
                    Block::BASALT.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                world.sync_world_event(WorldEvent::LavaFizz, *block_pos, 0);
                return false;
            }
        }
        true
    }
}

const LAVA_FLOW_SPEED_NETHER: u8 = 10;
const LAVA_FLOW_SPEED_SLOW: u8 = 30;

impl FluidBehaviour for FlowingLava {
    fn placed(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id != state_id && Self::receive_neighbor_fluids(world, block_pos) {
            let flow_speed = self.get_flow_speed(world);
            world.schedule_fluid_tick(fluid, *block_pos, flow_speed, TickPriority::Normal);
        }
    }

    fn on_scheduled_tick(&self, world: &Arc<World>, _fluid: &Fluid, block_pos: &BlockPos) {
        Self.on_scheduled_tick_internal(world, &Fluid::FLOWING_LAVA, block_pos);
    }

    fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: &BlockPos,
        _notify: bool,
    ) {
        if Self::receive_neighbor_fluids(world, block_pos) {
            let flow_speed = self.get_flow_speed(world);
            world.schedule_fluid_tick(fluid, *block_pos, flow_speed, TickPriority::Normal);
        }
    }

    fn on_entity_collision(&self, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        if !base_entity.is_fire_immune() {
            entity.set_on_fire_for(15.0);

            // Also apply lava damage
            base_entity.damage(entity, 4.0, DamageType::LAVA);
        }
    }

    fn random_tick(&self, _fluid: &Fluid, world: &Arc<World>, block_pos: &BlockPos) {
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

                let Some(state_id) = world.get_block_state_id_if_loaded(&test_pos) else {
                    return;
                };

                let (block, state) = BlockState::from_id_with_block(state_id);
                if state.is_air() {
                    if Self::has_flammable_neighbours(world, &test_pos) {
                        Self::ignite_fire_if_possible(world, &test_pos, &test_pos);
                        return;
                    }
                } else if block.has_tag(&tag::Block::MINECRAFT_BLOCKS_LAVA_FIRE_SPREAD) {
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

                let Some(test_state) = world.get_block_state_if_loaded(&test_pos) else {
                    return;
                };

                let above_pos = test_pos.up();
                if test_state.burnable()
                    && world
                        .get_block_state_if_loaded(&above_pos)
                        .is_some_and(BlockState::is_air)
                {
                    Self::ignite_fire_if_possible(world, &above_pos, &test_pos);
                }
            }
        }
    }
}

impl FlowingFluid for FlowingLava {
    fn get_level_decrease_per_block(&self, world: &World) -> i32 {
        if world.dimension.fast_lava { 1 } else { 2 }
    }

    fn get_flow_speed(&self, world: &World) -> u8 {
        // EnvironmentAttributes.FAST_LAVA
        if world.dimension.fast_lava {
            LAVA_FLOW_SPEED_NETHER
        } else {
            LAVA_FLOW_SPEED_SLOW
        }
    }

    fn get_max_flow_distance(&self, world: &World) -> i32 {
        if world.dimension.fast_lava { 5 } else { 3 }
    }

    /// Determines if lava can convert to source blocks based on game rules.
    fn can_convert_to_source(&self, world: &Arc<World>) -> bool {
        world.level_info.load().game_rules.lava_source_conversion
    }

    fn spread_to(&self, world: &Arc<World>, fluid: &Fluid, pos: &BlockPos, state_id: BlockStateId) {
        let new_props = FlowingFluidProperties::from_state_id(state_id, fluid);
        let current_state_id = world.get_block_state_id(pos);
        let block = Block::from_state_id(current_state_id);

        if new_props.level == Level::L8 && new_props.falling == Falling::True {
            // Stone creation when lava meets water
            if block == &Block::WATER {
                world.set_block_state(pos, Block::STONE.default_state.id, BlockFlags::NOTIFY_ALL);
                world.sync_world_event(WorldEvent::LavaFizz, *pos, 0);
                return;
            }
        }

        // Don't flow into waterlogged blocks
        if block.is_waterlogged(current_state_id) {
            return;
        }

        // Delegate quiescence, replacement and scheduling to the shared helper
        self.apply_spread(world, fluid, pos, state_id, new_props);
    }
}
