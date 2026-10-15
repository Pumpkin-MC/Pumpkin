use std::sync::Arc;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, ParentHandle};
use crate::entity::mob::Mob;
use crate::entity::passive::cat::CatEntity;
use crate::entity::passive::tamable::TamableAnimal;
use crate::world::World;

/// Tamed cats seek out beds to lie down on.
/// Mirrors vanilla Minecraft's `CatLieOnBlockGoal`.
pub struct CatLieOnBlockGoal {
    cat: Arc<CatEntity>,
    pub move_to_target_pos_goal: MoveToTargetPosGoal<Self>,
}

impl CatLieOnBlockGoal {
    #[must_use]
    pub fn new(cat: Arc<CatEntity>, speed: f64, range: i32) -> Box<Self> {
        let mut this = Box::new(Self {
            cat,
            move_to_target_pos_goal: MoveToTargetPosGoal::new(
                ParentHandle::none(),
                speed,
                range,
                6,
            ),
        });
        // SAFETY: `this` is pinned in Box and outlives `ParentHandle` references.
        this.move_to_target_pos_goal.move_to_target_pos = unsafe { ParentHandle::new(&this) };
        this
    }
}

impl MoveToTargetPos for CatLieOnBlockGoal {
    fn is_target_pos(&self, world: Arc<World>, pos: BlockPos) -> bool {
        let above = pos.up();
        if !world.get_block_state(&above).is_air() {
            return false;
        }

        let block = world.get_block(&pos);
        block.has_tag(&tag::Block::MINECRAFT_BEDS)
    }
}

impl Goal for CatLieOnBlockGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.cat.is_tame()
            && !self.cat.is_sitting()
            && !self.cat.is_lying()
            && self.move_to_target_pos_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.move_to_target_pos_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.start(mob);
        self.cat.set_sitting(false);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.stop(mob);
        self.cat.set_lying(false);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob);
        self.cat.set_sitting(false);
        if !self.move_to_target_pos_goal.reached {
            self.cat.set_lying(false);
        } else if !self.cat.is_lying() {
            self.cat.set_lying(true);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}
