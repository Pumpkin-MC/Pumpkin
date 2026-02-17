use super::{pathfinder, physics};
use crate::plugin::block::fluid_level_change::FluidLevelChangeEvent;
use crate::{block::BlockFuture, world::World};
use pumpkin_data::{
    Block, BlockDirection,
    fluid::{EnumVariants, Falling, Fluid, FluidProperties, Level},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, tick::TickPriority, world::BlockFlags};
use std::sync::Arc;
pub type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;
pub type FluidFuture<'a, T> = BlockFuture<'a, T>;

pub trait FlowingFluid: Send + Sync {
    fn get_level_decrease_per_block(&self, world: &World) -> i32;
    fn get_flow_speed(&self, world: &World) -> u8;

    fn get_source(&self, fluid: &Fluid, falling: bool) -> FlowingFluidProperties {
        let mut source_props = FlowingFluidProperties::default(fluid);
        source_props.level = Level::L8;
        source_props.falling = if falling {
            Falling::True
        } else {
            Falling::False
        };
        source_props
    }

    fn get_flowing(&self, fluid: &Fluid, level: Level, falling: bool) -> FlowingFluidProperties {
        let mut flowing_props = FlowingFluidProperties::default(fluid);
        flowing_props.level = level;
        flowing_props.falling = if falling {
            Falling::True
        } else {
            Falling::False
        };
        flowing_props
    }

    fn get_max_flow_distance(&self, world: &World) -> i32;
    fn can_convert_to_source(&self, world: &Arc<World>) -> bool;

    fn is_same_fluid(&self, fluid: &Fluid, other_state_id: BlockStateId) -> bool {
        Fluid::from_state_id(other_state_id).is_some_and(|other| fluid.matches_type(other))
    }

    /// Core fluid tick handler that updates fluid state and triggers spreading.
    ///
    /// Processes scheduled fluid ticks by:
    /// 1. Validating the block contains fluid
    /// 2. Updating non-source fluid levels based on neighbors
    /// 3. Triggering fluid spread to adjacent positions
    ///
    /// Sources (level 8, non-falling) always spread without state changes.
    fn on_scheduled_tick_internal<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {
            //let block = world.get_block(block_pos).await;
            let current_block_state_id = world.get_block_state_id(block_pos).await;
            let block = Block::from_state_id(current_block_state_id);

            let waterlogged = block.is_waterlogged(current_block_state_id);
            let is_fluid_state_id = fluid
                .states
                .iter()
                .any(|state| state.block_state_id == current_block_state_id)
                || waterlogged;

            if !is_fluid_state_id {
                return;
            }

            let current_fluid_state = if waterlogged {
                self.get_source(fluid, false)
            } else {
                FlowingFluidProperties::from_state_id(current_block_state_id, fluid)
            };
            let is_source = current_fluid_state.level == Level::L8
                && current_fluid_state.falling != Falling::True;
            let state_for_spreading: FlowingFluidProperties;

            // Update state if non-source
            if !is_source && !waterlogged {
                let new_fluid_state = self.get_new_liquid(world, fluid, block_pos).await;

                if let Some(new_state) = new_fluid_state {
                    let new_state_id = new_state.to_state_id(fluid);

                    if new_state_id != current_block_state_id {
                        if let Some(target_state_id) =
                            fire_fluid_level_change(world, block_pos, new_state_id).await
                        {
                            world
                                .set_block_state(block_pos, target_state_id, BlockFlags::NOTIFY_ALL)
                                .await;
                        } else {
                            return;
                        }

                        // Schedule next tick for this position
                        let tick_delay = self.get_flow_speed(world);
                        world
                            .schedule_fluid_tick(
                                fluid,
                                *block_pos,
                                tick_delay,
                                TickPriority::Normal,
                            )
                            .await;
                    }

                    // Use the new state for spreading
                    state_for_spreading = new_state;
                } else {
                    if !waterlogged
                        && let Some(target_state_id) =
                            fire_fluid_level_change(world, block_pos, Block::AIR.default_state.id)
                                .await
                    {
                        world
                            .set_block_state(block_pos, target_state_id, BlockFlags::NOTIFY_ALL)
                            .await;
                    }
                    return; // Don't spread if fluid is gone
                }
            } else {
                // Sources use their current state
                state_for_spreading = current_fluid_state;
            }

            // Then, spread using the appropriate state
            self.try_flow(world, fluid, block_pos, &state_for_spreading)
                .await;
        }
    }

    /// Attempts to flow fluid from a position, prioritizing downward flow.
    ///
    /// Flow priority:
    /// 1. Down - if space below, create falling fluid (level 8)
    /// 2. Sides - spread horizontally using pathfinding
    ///
    /// Sources with 3+ adjacent sources also spread to sides when flowing down.
    fn try_flow<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
        props: &'a FlowingFluidProperties,
    ) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {
            let below_pos = block_pos.down();
            let below_state = world.get_block_state(&below_pos).await;
            let below_block = Block::from_state_id(below_state.id);
            let is_hole = physics::can_be_replaced(below_state, below_block, fluid);

            // Try to flow down first
            if is_hole {
                let mut target_pos = below_pos;
                if let Some(server) = world.server.upgrade() {
                    let source_block = world.get_block(block_pos).await;
                    let to_block = world.get_block(&target_pos).await;
                    let event = crate::plugin::block::block_from_to::BlockFromToEvent::new(
                        source_block,
                        *block_pos,
                        to_block,
                        target_pos,
                        BlockDirection::Down,
                        world.uuid,
                    );
                    let event = server.plugin_manager.fire(event).await;
                    if event.cancelled {
                        return;
                    }
                    target_pos = event.to_pos;
                    let target_state = world.get_block_state(&target_pos).await;
                    let target_block = Block::from_state_id(target_state.id);
                    if !physics::can_be_replaced(target_state, target_block, fluid) {
                        return;
                    }
                }
                let falling_props = self.get_flowing(fluid, Level::L8, true);
                self.spread_to(world, fluid, &target_pos, falling_props.to_state_id(fluid))
                    .await;

                // Check if we should also spread to sides
                if props.level == Level::L8 && props.falling == Falling::False {
                    let source_count = self.count_source_neighbors(world, fluid, block_pos).await;
                    if source_count >= 3 {
                        self.flow_to_sides(world, fluid, block_pos, props).await;
                    }
                }
                return;
            }

            // Check if fluid should flow to the side(s)
            self.flow_to_sides(world, fluid, block_pos, props).await;
        }
    }

    fn count_source_neighbors<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> impl std::future::Future<Output = i32> + Send + 'a {
        async move {
            let mut count = 0;
            for direction in [
                BlockDirection::North,
                BlockDirection::South,
                BlockDirection::West,
                BlockDirection::East,
            ] {
                let neighbor_pos = block_pos.offset(direction.to_offset());
                let neighbor_id = world.get_block_state_id(&neighbor_pos).await;
                if self.is_same_fluid(fluid, neighbor_id) {
                    let props = FlowingFluidProperties::from_state_id(neighbor_id, fluid);
                    if props.level == Level::L8 && props.falling == Falling::False {
                        count += 1;
                    }
                }
            }
            count
        }
    }

    /// Calculates the new fluid state for a position based on neighbors and environment.
    ///
    /// Priority order:
    /// 1. Sources remain unchanged
    /// 2. Infinite source formation (2+ adjacent sources + solid/source below)
    /// 3. Fluid above forces falling state (level 8, falling)
    /// 4. Standard flow calculation from highest neighbor minus dropoff
    ///
    /// # Returns
    /// New fluid properties, or None if fluid should drain
    fn get_new_liquid<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> impl std::future::Future<Output = Option<FlowingFluidProperties>> + Send + 'a {
        async move {
            let current_state_id = world.get_block_state_id(block_pos).await;
            let current_props = FlowingFluidProperties::from_state_id(current_state_id, fluid);

            // Sources never change
            if current_props.level == Level::L8 && current_props.falling != Falling::True {
                return Some(current_props);
            }

            // First: check horizontal neighbors for infinite source formation
            let mut highest_neighbor = 0;
            let mut neighbor_source_count = 0;
            for direction in [
                BlockDirection::North,
                BlockDirection::South,
                BlockDirection::West,
                BlockDirection::East,
            ] {
                let neighbor_pos = block_pos.offset(direction.to_offset());
                let neighbor_state_id = world.get_block_state_id(&neighbor_pos).await;
                if !self.is_same_fluid(fluid, neighbor_state_id) {
                    continue;
                }

                let neighbor_props =
                    FlowingFluidProperties::from_state_id(neighbor_state_id, fluid);

                // Count horizontal non-falling sources for infinite source formation
                if neighbor_props.level == Level::L8 && neighbor_props.falling == Falling::False {
                    neighbor_source_count += 1;
                }

                // Falling water from the side counts as level 8
                let neighbor_level = if neighbor_props.falling == Falling::True {
                    8
                } else {
                    i32::from(neighbor_props.level.to_index()) + 1
                };

                highest_neighbor = highest_neighbor.max(neighbor_level);
            }

            // Attempt infinite source formation first
            if self.can_convert_to_source(world) && neighbor_source_count >= 2 {
                let below_pos = block_pos.down();
                let below_state = world.get_block_state(&below_pos).await;
                let below_state_id = below_state.id;

                // Check if block below is a stable source of the same fluid
                let below_is_same_source = if self.is_same_fluid(fluid, below_state_id) {
                    let below_props = FlowingFluidProperties::from_state_id(below_state_id, fluid);
                    below_props.level == Level::L8 && below_props.falling == Falling::False
                } else {
                    false
                };

                // If the block below is solid (solid block) or a source of same fluid, form a source here.
                if below_is_same_source || below_state.is_solid_block() {
                    return Some(self.get_source(fluid, false));
                }
                // Otherwise continue to standard falling/flowing logic
            }

            // Then: if there's water above, this block is ALWAYS level 8, falling=true
            let above_pos = block_pos.up();
            let above_state_id = world.get_block_state_id(&above_pos).await;
            let above_block = Block::from_state_id(above_state_id);

            if self.is_same_fluid(fluid, above_state_id)
                || above_block.is_waterlogged(above_state_id)
            {
                return Some(self.get_flowing(fluid, Level::L8, true));
            }

            // Standard flowing calculation
            let drop_off = self.get_level_decrease_per_block(world);
            let new_level = highest_neighbor - drop_off;

            if new_level <= 0 {
                None
            } else {
                Some(self.get_flowing(fluid, Level::from_index(new_level as u16 - 1), false))
            }
        }
    }

    /// Core spread logic with quiescence checks and state updates.
    ///
    /// Implements:
    /// - Quiescence: prevents unnecessary updates (e.g., source blocks, lower levels)
    /// - Infinite source formation checks (before and after placement)
    /// - Block replacement for non-fluid blocks
    /// - Fluid tick scheduling for non-source blocks
    ///
    /// Called by `spread_to` implementations after fluid-specific pre-checks.
    fn apply_spread<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        pos: &'a BlockPos,
        state_id: BlockStateId,
        new_props: FlowingFluidProperties,
    ) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {
            let current_state_id = world.get_block_state_id(pos).await;
            let is_already_same_fluid = self.is_same_fluid(fluid, current_state_id);
            if is_already_same_fluid {
                let current_props = FlowingFluidProperties::from_state_id(current_state_id, fluid);

                let current_level = i32::from(current_props.level.to_index()) + 1;
                let new_level = i32::from(new_props.level.to_index()) + 1;
                let current_is_source =
                    current_props.level == Level::L8 && current_props.falling == Falling::False;
                let new_is_source =
                    new_props.level == Level::L8 && new_props.falling == Falling::False;

                // Never overwrite a source with anything
                if current_is_source {
                    return;
                }

                // Check for infinite source formation before quiescence checks
                if !current_is_source && self.can_convert_to_source(world) {
                    let should_convert = self
                        .check_infinite_source_formation(world, fluid, pos)
                        .await;

                    if should_convert {
                        let source_props = self.get_source(fluid, false);
                        let source_state_id = source_props.to_state_id(fluid);
                        if let Some(target_state_id) =
                            fire_fluid_level_change(world, pos, source_state_id).await
                        {
                            world
                                .set_block_state(pos, target_state_id, BlockFlags::NOTIFY_ALL)
                                .await;
                        } else {
                            return;
                        }

                        // Sources don't need ticks
                        return;
                    }
                }

                // If new is a source, always accept it
                if new_is_source {
                    // Continue to set state below
                } else if current_props.falling == new_props.falling {
                    // Same falling state - check level
                    if new_level <= current_level {
                        return;
                    }
                }
            } else {
                // Replace non-fluid blocks
                let block = world.get_block(pos).await;
                if block.id != Block::AIR.id {
                    world.break_block(pos, None, BlockFlags::NOTIFY_ALL).await;
                }
            }

            if let Some(target_state_id) = fire_fluid_level_change(world, pos, state_id).await {
                world
                    .set_block_state(pos, target_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            } else {
                return;
            }

            // Check for infinite source formation after placing new fluid
            if self.can_convert_to_source(world) {
                let should_convert = self
                    .check_infinite_source_formation(world, fluid, pos)
                    .await;

                if should_convert {
                    let source_props = self.get_source(fluid, false);
                    let source_state_id = source_props.to_state_id(fluid);
                    if let Some(target_state_id) =
                        fire_fluid_level_change(world, pos, source_state_id).await
                    {
                        world
                            .set_block_state(pos, target_state_id, BlockFlags::NOTIFY_ALL)
                            .await;
                    } else {
                        return;
                    }

                    // Sources don't need ticks
                    return;
                }
            }

            // Only schedule tick if not a source
            let is_source = new_props.level == Level::L8 && new_props.falling == Falling::False;

            if !is_source {
                let tick_delay = self.get_flow_speed(world);
                world
                    .schedule_fluid_tick(fluid, *pos, tick_delay, TickPriority::Normal)
                    .await;
            }
        }
    }

    /// Checks if infinite source formation conditions are met.
    ///
    /// Requirements:
    /// - 2+ horizontally adjacent source blocks (level 8, non-falling)
    /// - Block below is either solid OR a source of the same fluid
    ///
    /// # Returns
    /// `true` if position should convert to a source block
    fn check_infinite_source_formation<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        pos: &'a BlockPos,
    ) -> impl std::future::Future<Output = bool> + Send + 'a {
        async move {
            // Count adjacent horizontal source blocks
            let mut source_count = 0;
            for direction in [
                BlockDirection::North,
                BlockDirection::South,
                BlockDirection::West,
                BlockDirection::East,
            ] {
                let neighbor_pos = pos.offset(direction.to_offset());
                let neighbor_state_id = world.get_block_state_id(&neighbor_pos).await;

                if self.is_same_fluid(fluid, neighbor_state_id) {
                    let neighbor_props =
                        FlowingFluidProperties::from_state_id(neighbor_state_id, fluid);

                    if neighbor_props.level == Level::L8 && neighbor_props.falling == Falling::False
                    {
                        source_count += 1;
                    }
                }
            }

            // Need at least 2 source neighbors
            if source_count < 2 {
                return false;
            }

            // Check the block below
            let below_pos = pos.down();
            let below_state = world.get_block_state(&below_pos).await;
            let below_state_id = below_state.id;

            // Check if block below is a stable source of the same fluid
            let below_is_same_source = if self.is_same_fluid(fluid, below_state_id) {
                let below_props = FlowingFluidProperties::from_state_id(below_state_id, fluid);

                below_props.level == Level::L8 && below_props.falling == Falling::False
            } else {
                false
            };

            // Convert to source if below is solid or a source of same fluid
            below_is_same_source || below_state.is_solid_block()
        }
    }

    /// Spreads fluid to a target position with the given state.
    ///
    /// Default implementation delegates to `apply_spread`. Implementations like
    /// lava can override to add fluid-specific logic (e.g., water -> stone conversion).
    fn spread_to<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        pos: &'a BlockPos,
        state_id: BlockStateId,
    ) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {
            let new_props = FlowingFluidProperties::from_state_id(state_id, fluid);
            self.apply_spread(world, fluid, pos, state_id, new_props)
                .await;
        }
    }

    /// Spreads fluid horizontally to adjacent positions using pathfinding.
    ///
    /// Uses `get_spread` to find optimal flow directions (shortest distance to holes)
    /// and the computed fluid state for each target position.
    fn flow_to_sides<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
        props: &'a FlowingFluidProperties,
    ) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {
            let drop_off = self.get_level_decrease_per_block(world);
            let current_level = i32::from(props.level.to_index()) + 1;
            let effective_level = if props.falling == Falling::True {
                7
            } else {
                current_level - drop_off
            };

            if effective_level <= 0 {
                return;
            }

            let (spread_dirs, count) = pathfinder::get_spread(self, world, fluid, block_pos).await;

            for &(direction, state_id) in spread_dirs.iter().take(count) {
                let side_pos = block_pos.offset(direction.to_offset());

                // Re-verify the target position is still replaceable right before spreading
                let side_state = world.get_block_state(&side_pos).await;
                let side_block = Block::from_state_id(side_state.id);

                if !physics::can_be_replaced(side_state, side_block, fluid) {
                    continue;
                }

                let mut target_pos = side_pos;
                if let Some(server) = world.server.upgrade() {
                    let source_block = world.get_block(block_pos).await;
                    let to_block = world.get_block(&target_pos).await;
                    let event = crate::plugin::block::block_from_to::BlockFromToEvent::new(
                        source_block,
                        *block_pos,
                        to_block,
                        target_pos,
                        direction,
                        world.uuid,
                    );
                    let event = server.plugin_manager.fire(event).await;
                    if event.cancelled {
                        continue;
                    }
                    target_pos = event.to_pos;
                    let target_state = world.get_block_state(&target_pos).await;
                    let target_block = Block::from_state_id(target_state.id);
                    if !physics::can_be_replaced(target_state, target_block, fluid) {
                        continue;
                    }
                }

                let final_state_id = if target_pos == side_pos {
                    state_id
                } else {
                    let Some(new_props) = self.get_new_liquid(world, fluid, &target_pos).await
                    else {
                        continue;
                    };
                    new_props.to_state_id(fluid)
                };

                self.spread_to(world, fluid, &target_pos, final_state_id)
                    .await;
            }
        }
    }
}

async fn fire_fluid_level_change(
    world: &Arc<World>,
    pos: &BlockPos,
    new_state_id: BlockStateId,
) -> Option<BlockStateId> {
    let Some(server) = world.server.upgrade() else {
        return Some(new_state_id);
    };

    let block = world.get_block(pos).await;
    let event = FluidLevelChangeEvent::new(block, *pos, world.uuid, new_state_id);
    let event = server.plugin_manager.fire(event).await;
    if event.cancelled {
        None
    } else {
        Some(event.new_state_id)
    }
}
