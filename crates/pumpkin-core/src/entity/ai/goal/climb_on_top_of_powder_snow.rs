use super::{Controls, Goal, GoalReach};
use crate::entity::mob::Mob;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};

pub struct ClimbOnTopOfPowderSnowGoal;

impl Goal for ClimbOnTopOfPowderSnowGoal {
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach::default())
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        if !(entity
            .was_in_powder_snow
            .load(std::sync::atomic::Ordering::Relaxed)
            || entity.is_in_powder_snow())
            || !entity
                .entity_type
                .has_tag(&tag::EntityType::MINECRAFT_POWDER_SNOW_WALKABLE_MOBS)
        {
            return false;
        }

        let above = entity.block_pos.load().up();
        let world = entity.world.load();
        let above_state = world.get_block_state(&above);
        world.get_block(&above) == &Block::POWDER_SNOW
            || above_state.get_block_collision_shapes().next().is_none()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .jump_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .jump();
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::JUMP
    }
}
