use std::{collections::HashMap, i32};

use async_trait::async_trait;
use pumpkin_data::{block::Block, fluid::{Falling, Fluid, FluidProperties, Level}};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;

use crate::world::{BlockFlags, World};
type FlowingFluidProperties = pumpkin_data::fluid::FlowingWaterLikeFluidProperties;


fn level_to_int(level: Level) -> i32 {
    match level {
        Level::L1 => 1,
        Level::L2 => 2,
        Level::L3 => 3,
        Level::L4 => 4,
        Level::L5 => 5,
        Level::L6 => 6,
        Level::L7 => 7,
        Level::L8 => 8,
    }
}

fn int_to_level(level: i32) -> Level {
    match level {
        1 => Level::L1,
        2 => Level::L2,
        3 => Level::L3,
        4 => Level::L4,
        5 => Level::L5,
        6 => Level::L6,
        7 => Level::L7,
        8 => Level::L8,
        _ => Level::L1,
    }
}

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

        pub async fn is_hole<T: FlowingFluid + ?Sized + std::marker::Sync>(&mut self, fluid: &T, world: &World, fluid_type: &Fluid, pos: &BlockPos) -> bool {
        if let Some(is_hole) = self.holes.get(pos) {
            return *is_hole;
        }

        let below_pos = pos.down();
        let is_hole = fluid.is_water_hole(world, fluid_type, pos, &below_pos).await;

        self.holes.insert(pos.clone(), is_hole);
        is_hole
    }
}

#[async_trait]
pub trait FlowingFluid {
    async fn get_drop_off(&self) -> i32;

    async fn get_source(&self, fluid: &Fluid, falling: bool) -> FlowingFluidProperties;


    async fn get_flowing(&self, fluid: &Fluid, level: Level, falling: bool) -> FlowingFluidProperties;

    async fn get_slope_find_distance(&self) -> i32;

    async fn can_convert_to_source(&self, world: &World) -> bool;

    fn is_same_fluid(&self, fluid: &Fluid, other_state_id: u16) -> bool;

    async fn spread_fluid(&self, world: &World, fluid: &Fluid, block_pos: &BlockPos) {
        let block_state_id = match world.get_block_state_id(block_pos).await {
            Ok(id) => id,
            Err(_) => return,
        };

        let props = FlowingFluidProperties::from_state_id(block_state_id, fluid);

        match self.get_new_liquid(world, fluid, block_pos).await {
            Some(new_fluid_state) => {
                if new_fluid_state.to_state_id(fluid) != block_state_id {
                    world.set_block_state(block_pos, new_fluid_state.to_state_id(fluid), BlockFlags::NOTIFY_ALL).await;
                }
            },
            None => (),
        }

        self.spread(world, fluid, block_pos, &props).await;
    }

    async fn spread(&self, world: &World, fluid: &Fluid, block_pos: &BlockPos, props: &FlowingFluidProperties) {
        if level_to_int(props.level) <= 0 {
            return;
        }

        let below_pos = block_pos.down();
        let below_can_replace = self.can_replace_block(world, &below_pos, fluid).await;

        if below_can_replace {
            let mut new_props = FlowingFluidProperties::default(fluid);
            new_props.level = Level::L8;
            new_props.falling = Falling::True;

            self.spread_to(world, fluid, &below_pos, new_props.to_state_id(fluid)).await;
        } else {
            if props.level == Level::L8 || !self.is_water_hole(world, fluid, block_pos, &below_pos).await {
                self.spread_to_sides(world, fluid, block_pos).await;
            }
        }
    }

    async fn get_new_liquid(&self, world: &World, fluid: &Fluid, block_pos: &BlockPos) -> Option<FlowingFluidProperties> {
        let mut highest_level = 0;
        let mut source_count = 0;

        for direction in BlockDirection::horizontal() {
            let neighbor_pos = block_pos.offset(direction.to_offset());
            let neighbor_state_id = match world.get_block_state_id(&neighbor_pos).await {
                Ok(id) => id,
                Err(_) => continue,
            };

            if !self.is_same_fluid(fluid, neighbor_state_id) {
                continue;
            }

            let neighbor_props = FlowingFluidProperties::from_state_id(neighbor_state_id, fluid);
            let neighbor_level = level_to_int(neighbor_props.level);

            if neighbor_level == 8 && neighbor_props.falling != Falling::True {
                source_count += 1;
            }

            highest_level = highest_level.max(neighbor_level);
        }

        if source_count >= 2 && self.can_convert_to_source(world).await {
            let below_pos = block_pos.down();
            let below_state_id = match world.get_block_state_id(&below_pos).await {
                Ok(id) => id,
                Err(_) => 0,
            };

            if self.is_solid_or_source(world, &below_pos, below_state_id, fluid).await {
                return Some(self.get_source(fluid, false).await);
            }
        }

        let above_pos = block_pos.up();
        let above_state_id = match world.get_block_state_id(&above_pos).await {
            Ok(id) => id,
            Err(_) => 0,
        };

        if self.is_same_fluid(fluid, above_state_id) {
            return Some(self.get_flowing(fluid, Level::L8, true).await);
        }

        let drop_off = self.get_drop_off().await;
        let new_level = highest_level - drop_off;

        if new_level <= 0 {
            return None;
        }

        return Some(self.get_flowing(fluid, int_to_level(new_level), false).await);
    }

    async fn is_solid_or_source(&self, world: &World, block_pos: &BlockPos, state_id: u16, fluid: &Fluid) -> bool {
        let block = match world.get_block(block_pos).await {
            Ok(block) => block,
            Err(_) => return false,
        };

        if block.id != 0 && self.can_be_replaced(world, block_pos, block.id).await {
            return true;
        }

        if self.is_same_fluid(fluid, state_id) {
            log::info!("Fluid is same as state id");
            let props = FlowingFluidProperties::from_state_id(state_id, fluid);
            return props.level == Level::L8 && props.falling != Falling::True;
        }

        false
    }

    async fn spread_to_sides(&self, world: &World, fluid: &Fluid, block_pos: &BlockPos) {
        let block_state_id = match world.get_block_state_id(block_pos).await {
            Ok(id) => id,
            Err(_) => return,
        };

        let props = FlowingFluidProperties::from_state_id(block_state_id, fluid);
        let level = level_to_int(props.level);

        let effective_level = if props.falling == Falling::True {
            8
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
                let new_props = self.get_flowing(fluid, int_to_level(new_level), false).await;
                self.spread_to(world, fluid, &side_pos, new_props.to_state_id(fluid)).await;
            }
        }
    }

    async fn get_spread(&self, world: &World, fluid: &Fluid, block_pos: &BlockPos) -> HashMap<BlockDirection, i32>{
        let mut result = HashMap::new();
        let mut min_dist = i32::MAX;
        let mut ctx = SpreadContext::new();

        for direction in BlockDirection::horizontal() {
            let side_pos = block_pos.offset(direction.to_offset());

            if !self.can_replace_block(world, &side_pos, fluid).await {
                continue;
            }

            let slope_dist  = if ctx.is_hole(self, world, fluid, &side_pos).await {
                0
            }else{
                self.get_slope_distance(world, fluid, side_pos.clone(), 1, direction.opposite()).await
            };
            if slope_dist < min_dist {
                min_dist = slope_dist;
            }
        }

        for direction in BlockDirection::horizontal() {
            let side_pos = block_pos.offset(direction.to_offset());

            if !self.can_replace_block(world, &side_pos, fluid).await {
                continue;
            }

            let slope_dist = if ctx.is_hole(self, world, fluid, &side_pos).await {
                0
            } else {
                self.get_slope_distance(world, fluid, side_pos.clone(), 1, direction.opposite()).await
            };

            if slope_dist <= min_dist {
                result.insert(direction, slope_dist);
            }
        }

        result
    }

    async fn get_slope_distance(& self, world: &World, fluid: &Fluid, block_pos: BlockPos, distance: i32, exclude_dir: BlockDirection) -> i32 {
        if distance >= self.get_slope_find_distance().await {
            return distance;
        }

        let mut min_dist = 1000;

        let mut ctx = SpreadContext::new();

        for direction in BlockDirection::horizontal() {
            if direction == exclude_dir {
                continue;
            }

            let next_pos = block_pos.offset(direction.to_offset());

            if !self.can_pass_through(world, fluid, &next_pos).await {
                continue;
            }

            if ctx.is_hole(self, world, fluid, &next_pos).await {
                return distance;
            }

            let next_dist = self.get_slope_distance(world, fluid, next_pos.clone(), distance + 1, direction.opposite()).await;

            min_dist = min_dist.min(next_dist);
        }
        min_dist
    }

    async fn spread_to(&self, world: &World, _fluid: &Fluid, pos: &BlockPos, state_id: u16) {
        //TODO Implement lava water mix

        let existing_block = world.get_block(pos).await.ok();
        if let Some(block) = existing_block {
            if block.id != 0 {
                self.before_destroying_block(&world, pos, block.id).await;
            }
        }

        world.set_block_state(pos, state_id, BlockFlags::NOTIFY_ALL).await;
    }

    async fn can_pass_through(&self, world: &World, fluid: &Fluid, pos: &BlockPos) -> bool {
        let state_id = match world.get_block_state_id(pos).await {
            Ok(id) => id,
            Err(_) => return false,
        };

        if self.is_same_fluid(fluid, state_id) {
            let props = FlowingFluidProperties::from_state_id(state_id, fluid);
            if props.level == Level::L8 && props.falling != Falling::True {
                return false;
            }
        }

        self.can_replace_block(world, pos, fluid).await
    }

    async fn can_replace_block(&self, world: &World, pos: &BlockPos, _fluid: &Fluid) -> bool {
        let block = match world.get_block(pos).await {
            Ok(block) => block,
            Err(_) => return false,
        };

        if self.can_be_replaced(world, pos, block.id).await {
            return true;
        }

        false
    }

    async fn can_be_replaced(&self, _world: &World, _pos: &BlockPos, block_id: u16) -> bool {
        //TODO Add specific block IDs here that aren't solid
        match block_id {
            0 => true,
            _ => false,
        }
    }

    async fn before_destroying_block(&self, _world: &World, _pos: &BlockPos, block_id: u16) {
        //TODO Add specific block IDs here that need to be handled before destroying
        match block_id {
            _ => (),
        }
    }

    async fn is_water_hole(&self, world: &World, fluid: &Fluid, _pos: &BlockPos, below_pos: &BlockPos) -> bool {
        let below_state_id = match world.get_block_state_id(below_pos).await {
            Ok(id) => id,
            Err(_) => return false,
        };

        let below_block = match world.get_block(below_pos).await {
            Ok(block) => block,
            Err(_) => return false,
        };

        if self.is_same_fluid(fluid, below_state_id) {
            return true;
        }

        if below_block.id == 0 || self.can_replace_block(world, below_pos, fluid).await {
            return true;
        }

        false
    }
}
