use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};

use pumpkin_data::{
    Block, BlockDirection,
    fluid::{EnumVariants, Falling, Fluid, FluidProperties, Level},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockId, BlockStateId, world::BlockFlags};

use crate::{block::BlockFuture, world::World};
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

#[derive(Clone)]
pub struct SpreadContext {
    holes: HashMap<BlockPos, bool>,
}

impl Default for SpreadContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SpreadContext {
    #[must_use]
    pub fn new() -> Self {
        Self {
            holes: HashMap::new(),
        }
    }
    pub async fn is_hole<T: FlowingFluid + ?Sized + Sync>(
        &mut self,
        fluid: &T,
        world: &Arc<World>,
        fluid_type: &Fluid,
        pos: &BlockPos,
    ) -> bool {
        if let Some(is_hole) = self.holes.get(pos) {
            return *is_hole;
        }

        let below_pos = pos.down();
        let is_hole = fluid
            .is_water_hole(world, fluid_type, pos, &below_pos)
            .await;

        self.holes.insert(*pos, is_hole);
        is_hole
    }
}

pub type FluidFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait FlowingFluid: Send + Sync {
    fn get_level_decrease_per_block(&self, world: &World) -> i32;

    fn get_source<'a>(
        &'a self,
        fluid: &'a Fluid,
        falling: bool,
    ) -> FluidFuture<'a, FlowingFluidProperties> {
        Box::pin(async move {
            let mut source_props = FlowingFluidProperties::default(fluid);
            source_props.level = Level::L8;
            source_props.falling = if falling {
                Falling::True
            } else {
                Falling::False
            };
            source_props
        })
    }

    fn get_flowing<'a>(
        &'a self,
        fluid: &'a Fluid,
        level: Level,
        falling: bool,
    ) -> FluidFuture<'a, FlowingFluidProperties> {
        Box::pin(async move {
            let mut flowing_props = FlowingFluidProperties::default(fluid);
            flowing_props.level = level;
            flowing_props.falling = if falling {
                Falling::True
            } else {
                Falling::False
            };
            flowing_props
        })
    }

    fn get_max_flow_distance(&self, world: &World) -> i32;

    fn can_convert_to_source(&self, world: &Arc<World>) -> bool;

    fn is_waterlogged<'a>(
        &'a self,
        world: &'a Arc<World>,
        pos: &'a BlockPos,
    ) -> FluidFuture<'a, Option<BlockStateId>> {
        Box::pin(async move {
            let block = world.get_block(pos).await;

            let state_id = world.get_block_state_id(pos).await;
            // Check if the block has waterlogged property and if it's true
            if let Some(properties) = block.properties(state_id)
                && properties
                    .to_props()
                    .iter()
                    .any(|(key, value)| *key == "waterlogged" && *value == "true")
            {
                return Some(state_id);
            }
            None
        })
    }

    fn is_same_fluid(&self, fluid: &Fluid, other_state_id: BlockStateId) -> bool {
        if let Some(other_fluid) = Fluid::from_state_id(other_state_id) {
            return fluid.id == other_fluid.id;
        }
        false
    }

    fn on_scheduled_tick_internal<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> FluidFuture<'a, ()> {
        Box::pin(async move {
            let current_block_state = world.get_block_state(block_pos).await;
            let current_fluid_state =
                FlowingFluidProperties::from_state_id(current_block_state.id, fluid);

            if current_fluid_state.level != Level::L8
                || current_fluid_state.falling == Falling::True
            {
                let new_fluid_state = self.get_new_liquid(world, fluid, block_pos).await;
                if let Some(new_fluid_state) = new_fluid_state {
                    if new_fluid_state.to_state_id(fluid) != current_block_state.id {
                        world
                            .set_block_state(
                                block_pos,
                                new_fluid_state.to_state_id(fluid),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                    }
                } else if self.is_waterlogged(world, block_pos).await.is_none() {
                    world
                        .set_block_state(
                            block_pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
            self.try_flow(world, fluid, block_pos, &current_fluid_state)
                .await;
        })
    }

    fn try_flow<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
        props: &'a FlowingFluidProperties,
    ) -> FluidFuture<'a, ()> {
        Box::pin(async move {
            let below_pos = block_pos.down();
            let below_can_replace = !self.can_flow_through(world, &below_pos, 0, fluid).await;

            if below_can_replace {
                let mut new_props = FlowingFluidProperties::default(fluid);
                new_props.level = Level::L8;
                new_props.falling = Falling::True;

                self.spread_to(world, fluid, &below_pos, new_props.to_state_id(fluid))
                    .await;
                if self
                    .count_neighboring_sources(world, fluid, block_pos)
                    .await
                    >= 3
                {
                    self.flow_to_sides(world, fluid, block_pos).await;
                }
            } else if props.level == Level::L8 && props.falling == Falling::False
                || !self
                    .is_water_hole(world, fluid, block_pos, &below_pos)
                    .await
            {
                self.flow_to_sides(world, fluid, block_pos).await;
            }
        })
    }

    fn count_neighboring_sources<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> FluidFuture<'a, i32> {
        Box::pin(async move {
            let mut source_count = 0;

            for direction in BlockDirection::horizontal() {
                let neighbor_pos = block_pos.offset(direction.to_offset());
                let neighbor_state_id = world.get_block_state_id(&neighbor_pos).await;

                if fluid.default_state_index == Fluid::WATER.default_state_index
                    && self.is_waterlogged(world, &neighbor_pos).await.is_some()
                {
                    source_count += 1;
                    continue;
                }

                if !self.is_same_fluid(fluid, neighbor_state_id) {
                    continue;
                }

                let neighbor_props =
                    FlowingFluidProperties::from_state_id(neighbor_state_id, fluid);
                let neighbor_level = i32::from(neighbor_props.level.to_index()) + 1;

                if neighbor_level == 8 && neighbor_props.falling != Falling::True {
                    source_count += 1;
                }
            }
            source_count
        })
    }

    fn get_new_liquid<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> FluidFuture<'a, Option<FlowingFluidProperties>> {
        Box::pin(async move {
            let current_state_id = world.get_block_state_id(block_pos).await;

            let current_props = FlowingFluidProperties::from_state_id(current_state_id, fluid);
            let current_level = i32::from(current_props.level.to_index()) + 1;
            if current_level == 8 && current_props.falling != Falling::True {
                return Some(current_props);
            }
            let mut highest_level = 0;
            let mut source_count = 0;

            for direction in BlockDirection::horizontal() {
                let neighbor_pos = block_pos.offset(direction.to_offset());
                let neighbor_state_id = world.get_block_state_id(&neighbor_pos).await;

                if fluid.default_state_index == Fluid::WATER.default_state_index
                    && self.is_waterlogged(world, &neighbor_pos).await.is_some()
                {
                    source_count += 1;
                    highest_level = highest_level.max(8);
                    continue;
                }

                if !self.is_same_fluid(fluid, neighbor_state_id) {
                    continue;
                }

                let neighbor_props =
                    FlowingFluidProperties::from_state_id(neighbor_state_id, fluid);
                let neighbor_level = i32::from(neighbor_props.level.to_index()) + 1;

                if neighbor_level == 8 && neighbor_props.falling != Falling::True {
                    source_count += 1;
                }

                highest_level = highest_level.max(neighbor_level);
            }

            if source_count >= 2 && self.can_convert_to_source(world) {
                let below_pos = block_pos.down();
                let below_state_id = world.get_block_state_id(&below_pos).await;
                if self
                    .can_flow_through(world, &below_pos, below_state_id, fluid)
                    .await
                {
                    return Some(self.get_source(fluid, false).await);
                }
            }

            let above_pos = block_pos.up();
            let above_state_id = world.get_block_state_id(&above_pos).await;

            if self.is_same_fluid(fluid, above_state_id)
                || self.is_waterlogged(world, &above_pos).await.is_some()
            {
                return Some(self.get_flowing(fluid, Level::L8, true).await);
            }

            let drop_off = self.get_level_decrease_per_block(world);
            let new_level = highest_level - drop_off;

            if new_level <= 0 {
                return None;
            }
            if new_level != current_level {
                return Some(
                    self.get_flowing(fluid, Level::from_index(new_level as u16 - 1), false)
                        .await,
                );
            }
            Some(current_props)
        })
    }

    fn can_flow_through<'a>(
        &'a self,
        world: &'a Arc<World>,
        block_pos: &'a BlockPos,
        state_id: BlockStateId,
        fluid: &'a Fluid,
    ) -> FluidFuture<'a, bool> {
        Box::pin(async move {
            if self.is_same_fluid(fluid, state_id) {
                let props = FlowingFluidProperties::from_state_id(state_id, fluid);
                if props.level == Level::L8 && props.falling != Falling::True {
                    return true;
                }
            }
            world
                .get_block_state(block_pos)
                .await
                .is_side_solid(BlockDirection::Up)
        })
    }

    fn flow_to_sides<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> FluidFuture<'a, ()> {
        Box::pin(async move {
            let block_state_id = world.get_block_state_id(block_pos).await;

            let props = FlowingFluidProperties::from_state_id(block_state_id, fluid);
            let drop_off = self.get_level_decrease_per_block(world);

            let level = i32::from(props.level.to_index()) - drop_off;

            let effective_level = if props.falling == Falling::True {
                7
            } else {
                level
            };
            if effective_level <= 0 {
                return;
            }

            let spread_dirs = self.get_spread(world, fluid, block_pos).await;

            for (direction, _slope_dist) in spread_dirs {
                let side_pos = block_pos.offset(direction.to_offset());

                if self.can_replace_block(world, &side_pos, fluid).await {
                    let new_props = self
                        .get_flowing(fluid, Level::from_index(effective_level as u16 - 1), false)
                        .await;
                    self.spread_to(world, fluid, &side_pos, new_props.to_state_id(fluid))
                        .await;
                }
            }
        })
    }

    fn get_spread<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: &'a BlockPos,
    ) -> FluidFuture<'a, HashMap<BlockDirection, i32>> {
        Box::pin(async move {
            let mut min_dist = 1000;
            let mut result = HashMap::new();
            let mut ctx = None;
            for direction in BlockDirection::horizontal() {
                let side_pos = block_pos.offset(direction.to_offset());
                let side_state_id = world.get_block_state_id(&side_pos).await;

                let side_props = FlowingFluidProperties::from_state_id(side_state_id, fluid);

                if !self.can_pass_through(world, fluid, &side_pos).await
                    || (side_props.level == Level::L8 && side_props.falling != Falling::True)
                {
                    continue;
                }

                if ctx.is_none() {
                    ctx = Some(SpreadContext::new());
                }

                let ctx_ref = ctx.as_mut().unwrap();

                let slope_dist = if ctx_ref.is_hole(self, world, fluid, &side_pos).await {
                    0
                } else {
                    self.get_in_flow_down_distance(
                        world,
                        fluid,
                        side_pos,
                        1,
                        direction.opposite(),
                        ctx_ref,
                    )
                    .await
                };

                if slope_dist < min_dist {
                    result.clear();
                }

                if slope_dist <= min_dist {
                    result.insert(direction, slope_dist);
                    min_dist = slope_dist;
                }
            }
            result
        })
    }

    fn get_in_flow_down_distance<'a, 'b>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        block_pos: BlockPos,
        distance: i32,
        exclude_dir: BlockDirection,
        ctx: &'b mut SpreadContext,
    ) -> BlockFuture<'b, i32>
    where
        'a: 'b,
    {
        Box::pin(async move {
            if distance > self.get_max_flow_distance(world) {
                return 1000;
            }

            let mut min_dist = 1000;

            for direction in BlockDirection::horizontal() {
                if direction == exclude_dir {
                    continue;
                }

                let next_pos = block_pos.offset(direction.to_offset());

                if !self.can_pass_through(world, fluid, &next_pos).await {
                    continue;
                }

                let next_state_id = world.get_block_state_id(&next_pos).await;

                if self.is_same_fluid(fluid, next_state_id) {
                    let next_props = FlowingFluidProperties::from_state_id(next_state_id, fluid);
                    if next_props.level == Level::L8 && next_props.falling == Falling::False {
                        return 1000;
                    }
                }

                if ctx.is_hole(self, world, fluid, &next_pos).await {
                    return distance;
                }

                let next_dist = self
                    .get_in_flow_down_distance(
                        world,
                        fluid,
                        next_pos,
                        distance + 1,
                        direction.opposite(),
                        ctx,
                    )
                    .await;

                min_dist = min_dist.min(next_dist);
            }
            min_dist
        })
    }

    fn spread_to<'a>(
        &'a self,
        world: &'a Arc<World>,
        _fluid: &'a Fluid,
        pos: &'a BlockPos,
        state_id: BlockStateId,
    ) -> FluidFuture<'a, ()> {
        Box::pin(async move {
            if self.is_waterlogged(world, pos).await.is_some() {
                return;
            }

            world
                .set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL)
                .await;
        })
    }

    fn can_pass_through<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        pos: &'a BlockPos,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let state_id = world.get_block_state_id(pos).await;

            if self.is_same_fluid(fluid, state_id) {
                return true;
            }

            self.can_replace_block(world, pos, fluid).await
        })
    }

    fn can_replace_block<'a>(
        &'a self,
        world: &'a Arc<World>,
        pos: &'a BlockPos,
        fluid: &'a Fluid,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let block = world.get_block(pos).await;
            self.can_be_replaced(world, pos, block.id, fluid).await
        })
    }

    fn can_be_replaced<'a>(
        &'a self,
        world: &'a Arc<World>,
        pos: &'a BlockPos,
        block_id: BlockId,
        fluid: &'a Fluid,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            // let block_state_id = world.get_block_state_id(pos).await;
            let block_state = world.get_block_state(pos).await;

            if let Some(other_fluid) = Fluid::from_state_id(block_state.id) {
                if fluid.id != other_fluid.id {
                    return true;
                }
                if other_fluid.is_source(block_state.id) && other_fluid.is_falling(block_state.id) {
                    return true;
                }
            }

            //TODO Add check for blocks that aren't solid
            matches!(block_id, 0)
        })
    }

    fn is_water_hole<'a>(
        &'a self,
        world: &'a Arc<World>,
        fluid: &'a Fluid,
        _pos: &'a BlockPos,
        below_pos: &'a BlockPos,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let below_state_id = world.get_block_state_id(below_pos).await;

            if self.is_same_fluid(fluid, below_state_id) {
                return true;
            }

            if self.can_replace_block(world, below_pos, fluid).await {
                return true;
            }

            false
        })
    }
}
