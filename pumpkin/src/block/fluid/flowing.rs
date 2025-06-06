use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    fluid::{EnumVariants, Falling, Fluid, FluidProperties, Level},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockId, BlockStateId, world::BlockFlags};

use crate::world::World;
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;

#[derive(Clone)]
pub struct SpreadContext {
    holes: HashMap<BlockPos, bool>,
}

impl SpreadContext {
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

#[async_trait]
pub trait FlowingFluid {
    async fn get_drop_off(&self) -> i32;

    async fn get_source(&self, fluid: &Fluid, falling: bool) -> FlowingFluidProperties {
        let mut source_props = FlowingFluidProperties::default(fluid);
        source_props.level = Level::L8;
        source_props.falling = if falling {
            Falling::True
        } else {
            Falling::False
        };
        source_props
    }

    async fn get_flowing(
        &self,
        fluid: &Fluid,
        level: Level,
        falling: bool,
    ) -> FlowingFluidProperties {
        let mut flowing_props = FlowingFluidProperties::default(fluid);
        flowing_props.level = level;
        flowing_props.falling = if falling {
            Falling::True
        } else {
            Falling::False
        };
        flowing_props
    }

    async fn get_slope_find_distance(&self) -> i32;

    async fn can_convert_to_source(&self, world: &Arc<World>) -> bool;

    fn is_same_fluid(&self, fluid: &Fluid, other_state_id: BlockStateId) -> bool {
        if let Some(other_fluid) = Fluid::from_state_id(other_state_id) {
            return fluid.id == other_fluid.id;
        }
        false
    }

    async fn spread_fluid(&self, world: &Arc<World>, fluid: &Fluid, block_pos: &BlockPos) {
        let block_state_id = world.get_block_state_id(block_pos).await;
        if let Some(new_fluid_state) = self.get_new_liquid(world, fluid, block_pos).await {
            if new_fluid_state.to_state_id(fluid) != block_state_id {
                world
                    .set_block_state(
                        block_pos,
                        new_fluid_state.to_state_id(fluid),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
            self.spread(world, fluid, block_pos, &new_fluid_state).await;
        } else {
            world
                .break_block(block_pos, None, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
            world
                .set_block_state(
                    block_pos,
                    Block::AIR.default_state_id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    async fn spread(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: &BlockPos,
        props: &FlowingFluidProperties,
    ) {
        let below_pos = block_pos.down();
        let below_can_replace = !self.is_solid_or_source(world, &below_pos, 0, fluid).await;

        if below_can_replace {
            let mut new_props = FlowingFluidProperties::default(fluid);
            new_props.level = Level::L8;
            new_props.falling = Falling::True;

            self.spread_to(world, fluid, &below_pos, new_props.to_state_id(fluid))
                .await;
        } else if props.level == Level::L8 && props.falling == Falling::False
            || !self
                .is_water_hole(world, fluid, block_pos, &below_pos)
                .await
        {
            self.spread_to_sides(world, fluid, block_pos).await;
        }
    }

    async fn get_new_liquid(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: &BlockPos,
    ) -> Option<FlowingFluidProperties> {
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

            if !self.is_same_fluid(fluid, neighbor_state_id) {
                continue;
            }

            let neighbor_props = FlowingFluidProperties::from_state_id(neighbor_state_id, fluid);
            let neighbor_level = i32::from(neighbor_props.level.to_index()) + 1;

            if neighbor_level == 8 && neighbor_props.falling != Falling::True {
                source_count += 1;
            }

            highest_level = highest_level.max(neighbor_level);
        }

        if source_count >= 2 && self.can_convert_to_source(world).await {
            let below_pos = block_pos.down();
            let below_state_id = world.get_block_state_id(&below_pos).await;
            if self
                .is_solid_or_source(world, &below_pos, below_state_id, fluid)
                .await
            {
                return Some(self.get_source(fluid, false).await);
            }
        }

        let above_pos = block_pos.up();
        let above_state_id = world.get_block_state_id(&above_pos).await;

        if self.is_same_fluid(fluid, above_state_id) {
            return Some(self.get_flowing(fluid, Level::L8, true).await);
        }

        let drop_off = self.get_drop_off().await;
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
    }

    async fn is_solid_or_source(
        &self,
        world: &Arc<World>,
        block_pos: &BlockPos,
        state_id: BlockStateId,
        fluid: &Fluid,
    ) -> bool {
        let block = world.get_block(block_pos).await;

        if block.id != 0 && !self.can_be_replaced(world, block_pos, block.id).await {
            return true;
        }

        if self.is_same_fluid(fluid, state_id) {
            let props = FlowingFluidProperties::from_state_id(state_id, fluid);
            return props.level == Level::L8 && props.falling != Falling::True;
        }

        false
    }

    async fn spread_to_sides(&self, world: &Arc<World>, fluid: &Fluid, block_pos: &BlockPos) {
        let block_state_id = world.get_block_state_id(block_pos).await;

        let props = FlowingFluidProperties::from_state_id(block_state_id, fluid);
        let level = i32::from(props.level.to_index()) + 1;

        let effective_level = if props.falling == Falling::True {
            7
        } else {
            level
        };

        let drop_off = self.get_drop_off().await;
        let new_level = effective_level - drop_off;

        if new_level <= 0 {
            return;
        }

        let spread_dirs = self.get_spread(world, fluid, block_pos).await;

        for (direction, _slope_dist) in spread_dirs {
            let side_pos = block_pos.offset(direction.to_offset());

            if self.can_replace_block(world, &side_pos, fluid).await {
                let new_props = self
                    .get_flowing(fluid, Level::from_index(new_level as u16 - 1), false)
                    .await;
                self.spread_to(world, fluid, &side_pos, new_props.to_state_id(fluid))
                    .await;
            }
        }
    }

    async fn get_spread(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: &BlockPos,
    ) -> HashMap<BlockDirection, i32> {
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
                self.get_slope_distance(world, fluid, side_pos, 1, direction.opposite(), ctx_ref)
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
    }

    async fn get_slope_distance(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        block_pos: BlockPos,
        distance: i32,
        exclude_dir: BlockDirection,
        ctx: &mut SpreadContext,
    ) -> i32 {
        if distance > self.get_slope_find_distance().await {
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
                .get_slope_distance(
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
    }

    async fn spread_to(
        &self,
        world: &Arc<World>,
        _fluid: &Fluid,
        pos: &BlockPos,
        state_id: BlockStateId,
    ) {
        //TODO Implement lava water mix

        world
            .set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL)
            .await;
    }

    async fn can_pass_through(&self, world: &Arc<World>, fluid: &Fluid, pos: &BlockPos) -> bool {
        let state_id = world.get_block_state_id(pos).await;

        if self.is_same_fluid(fluid, state_id) {
            return true;
        }

        self.can_replace_block(world, pos, fluid).await
    }

    async fn can_replace_block(&self, world: &Arc<World>, pos: &BlockPos, _fluid: &Fluid) -> bool {
        let block = world.get_block(pos).await;

        if self.can_be_replaced(world, pos, block.id).await {
            return true;
        }

        false
    }

    async fn can_be_replaced(&self, world: &Arc<World>, pos: &BlockPos, block_id: BlockId) -> bool {
        let block_state_id = world.get_block_state_id(pos).await;

        if let Some(fluid) = Fluid::from_state_id(block_state_id) {
            if fluid.is_source(block_state_id) && fluid.is_falling(block_state_id) {
                return true;
            }
        }

        //TODO Add check for blocks that aren't solid
        matches!(block_id, 0)
    }

    async fn is_water_hole(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        _pos: &BlockPos,
        below_pos: &BlockPos,
    ) -> bool {
        let below_state_id = world.get_block_state_id(below_pos).await;

        if self.is_same_fluid(fluid, below_state_id) {
            return true;
        }

        if self.can_replace_block(world, below_pos, fluid).await {
            return true;
        }

        false
    }
}
