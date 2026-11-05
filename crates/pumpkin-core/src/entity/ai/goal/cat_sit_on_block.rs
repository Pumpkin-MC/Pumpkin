use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::block_properties::{BedPart, FurnaceLikeProperties, WhiteBedLikeProperties};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;

use super::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use super::{Controls, Goal, ParentHandle};
use crate::block::entities::chest::ChestBlockEntity;
use crate::entity::mob::Mob;
use crate::entity::passive::cat::CatEntity;
use crate::entity::passive::tamable::TamableAnimal;
use crate::world::World;

const SEARCH_RANGE: i32 = 8;

struct CatSitOnBlockTarget;

static TARGET: CatSitOnBlockTarget = CatSitOnBlockTarget;

impl MoveToTargetPos for CatSitOnBlockTarget {
    fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool {
        if !world.get_block_state(&block_pos.up()).is_air() {
            return false;
        }
        let (block, state) = world.get_block_and_state(&block_pos);
        if !block.has_tag(&tag::Block::MINECRAFT_CATS_CAN_SIT_ON) {
            return false;
        }
        if block == &Block::CHEST {
            return world
                .get_block_entity(&block_pos)
                .is_none_or(|block_entity| {
                    block_entity
                        .as_any()
                        .downcast_ref::<ChestBlockEntity>()
                        .is_none_or(|chest| chest.get_viewer_count() < 1)
                });
        }
        if block == &Block::FURNACE {
            return FurnaceLikeProperties::from_state_id(state.id).lit;
        }
        if block.has_tag(&tag::Block::MINECRAFT_BEDS) {
            return WhiteBedLikeProperties::from_state_id(state.id).part != BedPart::Head;
        }
        true
    }
}

pub struct CatSitOnBlockGoal {
    cat: Weak<CatEntity>,
    move_to_target_pos_goal: MoveToTargetPosGoal<CatSitOnBlockTarget>,
}

impl CatSitOnBlockGoal {
    #[must_use]
    pub fn new(cat: Weak<CatEntity>, speed: f64) -> Box<Self> {
        Box::new(Self {
            cat,
            move_to_target_pos_goal: MoveToTargetPosGoal::with_default(
                ParentHandle::from_static(&TARGET),
                speed,
                SEARCH_RANGE,
            ),
        })
    }
}

impl Goal for CatSitOnBlockGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(cat) = self.cat.upgrade() else {
            return false;
        };
        cat.is_tame() && !cat.is_ordered_to_sit() && self.move_to_target_pos_goal.can_start(mob)
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
            cat.set_in_sitting_pose(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob);
        if let Some(cat) = self.cat.upgrade() {
            cat.set_in_sitting_pose(self.move_to_target_pos_goal.is_reached_target());
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}
