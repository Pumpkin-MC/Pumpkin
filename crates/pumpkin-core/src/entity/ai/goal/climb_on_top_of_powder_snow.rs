use std::sync::atomic::Ordering::{Relaxed, SeqCst};

use pumpkin_data::Block;
use pumpkin_data::tag::{self, Taggable};

use super::{Controls, Goal};
use crate::entity::mob::Mob;

pub struct ClimbOnTopOfPowderSnowGoal;

impl Goal for ClimbOnTopOfPowderSnowGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        if !entity.is_in_powder_snow() && !entity.was_in_powder_snow.load(Relaxed) {
            return false;
        }

        if !entity
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_POWDER_SNOW_WALKABLE_MOBS)
        {
            return false;
        }

        let above = entity.block_pos.load().up();
        let above_state = entity.world.load().get_block_state(&above);
        above_state.id.to_block() == &Block::POWDER_SNOW
            || above_state
                .get_block_collision_shapes_at(&above)
                .next()
                .is_none()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .living_entity
            .jumping
            .store(true, SeqCst);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .living_entity
            .jumping
            .store(false, SeqCst);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::JUMP
    }
}
