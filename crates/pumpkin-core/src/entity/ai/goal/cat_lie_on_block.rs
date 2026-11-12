use std::sync::Weak;

use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, ParentHandle};
use crate::entity::mob::Mob;
use crate::entity::passive::cat::CatEntity;
use crate::entity::passive::tamable::TamableAnimal;
use crate::world::World;

const NEXT_START_TICK: i32 = 40;

struct CatLieOnBlockTarget;

static TARGET: CatLieOnBlockTarget = CatLieOnBlockTarget;

impl MoveToTargetPos for CatLieOnBlockTarget {
    fn is_target_pos(&self, world: &World, block_pos: BlockPos) -> bool {
        world.get_block_state(&block_pos.up()).is_air()
            && world
                .get_block(&block_pos)
                .has_tag(&tag::Block::MINECRAFT_CATS_CAN_LIE_ON)
    }

    fn next_start_tick(&self, _mob: &dyn Mob) -> i32 {
        NEXT_START_TICK
    }
}

pub struct CatLieOnBlockGoal {
    cat: Weak<CatEntity>,
    move_to_target_pos_goal: MoveToTargetPosGoal<CatLieOnBlockTarget>,
}

impl CatLieOnBlockGoal {
    #[must_use]
    pub fn new(cat: Weak<CatEntity>, speed: f64, range: i32) -> Box<Self> {
        let mut move_to_target_pos_goal =
            MoveToTargetPosGoal::new(ParentHandle::from_static(&TARGET), speed, range, 6);
        move_to_target_pos_goal.lowest_y = -2;
        Box::new(Self {
            cat,
            move_to_target_pos_goal,
        })
    }
}

impl Goal for CatLieOnBlockGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(cat) = self.cat.upgrade() else {
            return false;
        };
        cat.is_tame()
            && !cat.is_ordered_to_sit()
            && !cat.is_lying()
            && self.move_to_target_pos_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.move_to_target_pos_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.start(mob);
        if let Some(cat) = self.cat.upgrade() {
            cat.set_in_sitting_pose(false);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.stop(mob);
        if let Some(cat) = self.cat.upgrade() {
            cat.set_lying(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob);
        let Some(cat) = self.cat.upgrade() else {
            return;
        };
        cat.set_in_sitting_pose(false);
        if !self.move_to_target_pos_goal.is_reached_target() {
            cat.set_lying(false);
        } else if !cat.is_lying() {
            cat.set_lying(true);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}
