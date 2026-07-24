use super::{pathfinder, physics};
use crate::{block::BlockFuture, world::World};
use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    fluid::{EnumVariants, Falling, Fluid, FluidProperties, Level},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use std::sync::Arc;
pub type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;
pub type FluidFuture<'a, T> = BlockFuture<'a, T>;

/// Vanilla `LiquidBlock` level base for water / lava block state ids.
/// Water: 86..=101, Lava: 102..=117.
fn fluid_block_base(fluid: &Fluid) -> Option<u16> {
    match fluid.id {
        // FLOWING_WATER / WATER
        1 | 2 => Some(86),
        // FLOWING_LAVA / LAVA
        3 | 4 => Some(102),
        _ => None,
    }
}

/// Vanilla `LiquidBlock.getFluidState` mapping:
/// ```text
/// level 0      → still source (amount 8, not falling)
/// level 1..=15 → flowing(amount = 8 - (level & 7), falling = level >= 8)
/// ```
/// So level 1 → amount 7, level 7 → amount 1, level 8 → amount 8 falling,
/// level 9 → amount 7 falling, level 15 → amount 1 falling.
fn props_from_block_level(fluid: &Fluid, block_level: u16) -> FlowingFluidProperties {
    let mut props = FlowingFluidProperties::default(fluid);
    if block_level == 0 {
        props.level = Level::L8;
        props.falling = Falling::False;
        return props;
    }
    let amount = 8 - (block_level & 7);
    props.level = Level::from_index(amount - 1);
    props.falling = if block_level >= 8 {
        Falling::True
    } else {
        Falling::False
    };
    props
}

/// Vanilla `FlowingFluid.getLegacyLevel` / `createLegacyBlock`:
/// - source → 0
/// - falling → 8 - min(amount, 8) + 8
/// - else → 8 - min(amount, 8)
fn block_level_from_props(props: FlowingFluidProperties) -> u16 {
    let amount = props.level.to_index() + 1; // L1=1 … L8=8
    let is_source = props.level == Level::L8 && props.falling == Falling::False;
    if is_source {
        0
    } else if props.falling == Falling::True {
        8 - amount.min(8) + 8
    } else {
        8 - amount.min(8)
    }
}

/// Convert fluid props → block state id using vanilla LiquidBlock levels.
/// The generated `FlowingWaterLikeFluidProperties::to_state_id` table is inverted
/// (source 86 was encoded as falling L8), so callers must use this helper.
fn props_to_state_id(fluid: &Fluid, props: FlowingFluidProperties) -> BlockStateId {
    if let Some(base) = fluid_block_base(fluid) {
        let level = block_level_from_props(props);
        return BlockStateId::new(base + level).expect("fluid block state id");
    }
    // Fallback for unexpected fluids
    props.to_state_id(fluid)
}

/// Parse a block state id into fluid props via vanilla LiquidBlock levels.
fn props_from_state_id(fluid: &Fluid, state_id: BlockStateId) -> Option<FlowingFluidProperties> {
    let base = fluid_block_base(fluid)?;
    let id = state_id.as_u16();
    if id >= base && id <= base + 15 {
        return Some(props_from_block_level(fluid, id - base));
    }
    None
}

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

    /// Block state id for these fluid properties (vanilla LiquidBlock level mapping).
    fn props_to_block_state(&self, fluid: &Fluid, props: FlowingFluidProperties) -> BlockStateId {
        props_to_state_id(fluid, props)
    }

    fn get_max_flow_distance(&self, world: &World) -> i32;
    fn can_convert_to_source(&self, world: &Arc<World>) -> bool;

    /// Returns true if `state_id` represents the given fluid — either as a direct fluid state
    /// or as a waterlogged block (when the fluid is water-type).
    fn has_fluid_at(&self, fluid: &Fluid, state_id: BlockStateId) -> bool {
        self.get_effective_props(fluid, state_id).is_some()
    }

    /// Returns correct fluid properties for a state, treating waterlogged blocks as water sources
    /// (level 8, non-falling). Returns `None` if the state doesn't contain this fluid.
    ///
    /// Uses vanilla `LiquidBlock` level encoding — **not** the inverted generated fluid
    /// property table (which mapped source state 86 to falling L8).
    fn get_effective_props(
        &self,
        fluid: &Fluid,
        state_id: BlockStateId,
    ) -> Option<FlowingFluidProperties> {
        if let Some(props) = props_from_state_id(fluid, state_id) {
            return Some(props);
        }
        if fluid.id == Fluid::FLOWING_WATER.id || fluid.id == Fluid::WATER.id {
            let block = Block::from_state_id(state_id);
            if block.is_waterlogged(state_id) {
                return Some(self.get_source(fluid, false));
            }
        }
        None
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
            //let block = world.get_block(block_pos);
            let current_block_state_id = world.get_block_state_id(block_pos);
            let block = Block::from_state_id(current_block_state_id);

            if !self.has_fluid_at(fluid, current_block_state_id) {
                return;
            }

            let waterlogged = block.is_waterlogged(current_block_state_id);
            let current_fluid_state = self
                .get_effective_props(fluid, current_block_state_id)
                .unwrap();
            let is_source = current_fluid_state.level == Level::L8
                && current_fluid_state.falling != Falling::True;
            let state_for_spreading: FlowingFluidProperties;

            // Update state if non-source
            if !is_source && !waterlogged {
                let new_fluid_state = self.get_new_liquid(world, fluid, block_pos).await;

                if let Some(new_state) = new_fluid_state {
                    let new_state_id = self.props_to_block_state(fluid, new_state);

                    if new_state_id != current_block_state_id {
                        world
                            .set_block_state(block_pos, new_state_id, BlockFlags::NOTIFY_ALL)
                            .await;

                        // Schedule next tick for this position
                        let tick_delay = self.get_flow_speed(world);
                        world.schedule_fluid_tick(
                            fluid,
                            *block_pos,
                            tick_delay,
                            TickPriority::Normal,
                        );
                    }

                    // Use the new state for spreading
                    state_for_spreading = new_state;
                } else {
                    if !waterlogged {
                        world
                            .set_block_state(
                                block_pos,
                                Block::AIR.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
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
            let below_state = world.get_block_state(&below_pos);
            let below_block = Block::from_state_id(below_state.id);
            let is_hole = physics::can_be_replaced(below_state, below_block, fluid);

            // Try to flow down first
            if is_hole {
                let falling_props = self.get_flowing(fluid, Level::L8, true);
                self.spread_to(
                    world,
                    fluid,
                    &below_pos,
                    self.props_to_block_state(fluid, falling_props),
                )
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
                let neighbor_id = world.get_block_state_id(&neighbor_pos);
                if self
                    .get_effective_props(fluid, neighbor_id)
                    .is_some_and(|p| p.level == Level::L8 && p.falling == Falling::False)
                {
                    count += 1;
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
            let current_state_id = world.get_block_state_id(block_pos);
            // Empty cells (air / non-fluid) have no "current" props — skip source early-out.
            // Critical: must use get_effective_props (vanilla level map), not the inverted
            // generated from_state_id which treated source state 86 as falling.
            if let Some(current_props) = self.get_effective_props(fluid, current_state_id) {
                // Sources never change
                if current_props.level == Level::L8 && current_props.falling != Falling::True {
                    return Some(current_props);
                }
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
                let neighbor_state_id = world.get_block_state_id(&neighbor_pos);
                let Some(neighbor_props) = self.get_effective_props(fluid, neighbor_state_id)
                else {
                    continue;
                };

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
                let below_state = world.get_block_state(&below_pos);
                let below_state_id = below_state.id;

                // Check if block below is a stable source of the same fluid
                let below_is_same_source = self
                    .get_effective_props(fluid, below_state_id)
                    .is_some_and(|p| p.level == Level::L8 && p.falling == Falling::False);

                // If the block below is solid (solid block) or a source of same fluid, form a source here.
                if below_is_same_source || below_state.is_solid_block() {
                    return Some(self.get_source(fluid, false));
                }
                // Otherwise continue to standard falling/flowing logic
            }

            // Then: if there's water above, this block is ALWAYS level 8, falling=true
            let above_pos = block_pos.up();
            let above_state_id = world.get_block_state_id(&above_pos);

            if self.has_fluid_at(fluid, above_state_id) {
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
        _state_id: BlockStateId,
        new_props: FlowingFluidProperties,
    ) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {
            let current_state_id = world.get_block_state_id(pos);
            if let Some(current_props) = self.get_effective_props(fluid, current_state_id) {
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
                        let source_state_id = self.props_to_block_state(fluid, source_props);
                        world
                            .set_block_state(pos, source_state_id, BlockFlags::NOTIFY_ALL)
                            .await;

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
                let block = world.get_block(pos);
                if block.id != Block::AIR.id {
                    world.break_block(pos, None, BlockFlags::NOTIFY_ALL).await;
                }
            }

            // Prefer props-derived state so source places as level 0 (state 86), not the
            // inverted generated table's level-8 flowing state.
            let place_id = self.props_to_block_state(fluid, new_props);
            world
                .set_block_state(pos, place_id, BlockFlags::NOTIFY_ALL)
                .await;

            // Check for infinite source formation after placing new fluid
            if self.can_convert_to_source(world) {
                let should_convert = self
                    .check_infinite_source_formation(world, fluid, pos)
                    .await;

                if should_convert {
                    let source_props = self.get_source(fluid, false);
                    let source_state_id = self.props_to_block_state(fluid, source_props);
                    world
                        .set_block_state(pos, source_state_id, BlockFlags::NOTIFY_ALL)
                        .await;

                    // Sources don't need ticks
                    return;
                }
            }

            // Only schedule tick if not a source
            let is_source = new_props.level == Level::L8 && new_props.falling == Falling::False;

            if !is_source {
                let tick_delay = self.get_flow_speed(world);
                world.schedule_fluid_tick(fluid, *pos, tick_delay, TickPriority::Normal);
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
                let neighbor_state_id = world.get_block_state_id(&neighbor_pos);

                if self
                    .get_effective_props(fluid, neighbor_state_id)
                    .is_some_and(|p| p.level == Level::L8 && p.falling == Falling::False)
                {
                    source_count += 1;
                }
            }

            // Need at least 2 source neighbors
            if source_count < 2 {
                return false;
            }

            // Check the block below
            let below_pos = pos.down();
            let below_state = world.get_block_state(&below_pos);
            let below_state_id = below_state.id;

            // Check if block below is a stable source of the same fluid
            let below_is_same_source = self
                .get_effective_props(fluid, below_state_id)
                .is_some_and(|p| p.level == Level::L8 && p.falling == Falling::False);

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
            let new_props = self
                .get_effective_props(fluid, state_id)
                .unwrap_or_else(|| FlowingFluidProperties::default(fluid));
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

                self.spread_to(world, fluid, &side_pos, state_id).await;
            }
        }
    }
}

#[cfg(test)]
mod liquid_block_level_tests {
    use super::*;
    use pumpkin_data::fluid::Fluid;

    fn roundtrip(block_level: u16) {
        let fluid = &Fluid::FLOWING_WATER;
        let props = props_from_block_level(fluid, block_level);
        let back = block_level_from_props(props);
        assert_eq!(
            back, block_level,
            "block level {block_level} → props → {back}"
        );
    }

    #[test]
    fn source_is_block_level_0_state_86() {
        let fluid = &Fluid::FLOWING_WATER;
        let source = {
            let mut p = FlowingFluidProperties::default(fluid);
            p.level = Level::L8;
            p.falling = Falling::False;
            p
        };
        assert_eq!(block_level_from_props(source), 0);
        assert_eq!(
            props_to_state_id(fluid, source),
            Block::WATER.default_state.id
        );

        // Bucket / default water must read as source, not falling.
        let from_default = props_from_state_id(fluid, Block::WATER.default_state.id).unwrap();
        assert_eq!(from_default.level, Level::L8);
        assert_eq!(from_default.falling, Falling::False);
    }

    #[test]
    fn vanilla_level_roundtrip_0_to_15() {
        for level in 0..=15u16 {
            roundtrip(level);
        }
    }

    #[test]
    fn flowing_heights_match_vanilla() {
        let fluid = &Fluid::FLOWING_WATER;
        // level 1 → amount 7 non-falling; level 7 → amount 1
        let p1 = props_from_block_level(fluid, 1);
        assert_eq!(p1.level, Level::L7);
        assert_eq!(p1.falling, Falling::False);

        let p7 = props_from_block_level(fluid, 7);
        assert_eq!(p7.level, Level::L1);
        assert_eq!(p7.falling, Falling::False);

        // level 8 → amount 8 falling
        let p8 = props_from_block_level(fluid, 8);
        assert_eq!(p8.level, Level::L8);
        assert_eq!(p8.falling, Falling::True);

        // level 9 → amount 7 falling
        let p9 = props_from_block_level(fluid, 9);
        assert_eq!(p9.level, Level::L7);
        assert_eq!(p9.falling, Falling::True);
    }
}
