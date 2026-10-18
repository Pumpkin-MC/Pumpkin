use std::sync::atomic::Ordering;

use pumpkin_data::Block;

use super::{Controls, Goal};
use crate::entity::mob::Mob;

pub struct ClimbOnTopOfPowderSnowGoal {
    goal_control: Controls,
}

impl Default for ClimbOnTopOfPowderSnowGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::JUMP,
        }
    }
}

impl ClimbOnTopOfPowderSnowGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::JUMP,
        }
    }
}

impl Goal for ClimbOnTopOfPowderSnowGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        let block = world.get_block_state(&pos).id.to_block();
        if block.id == Block::POWDER_SNOW.id {
            let above = pos.up();
            let above_state = world.get_block_state(&above);
            let above_block = above_state.id.to_block();
            above_block.id == Block::POWDER_SNOW.id || above_state.collision_shapes.is_empty()
        } else {
            false
        }
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn tick(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .living_entity
            .jumping
            .store(true, Ordering::SeqCst);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
