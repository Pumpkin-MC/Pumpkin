use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use crate::entity::mob::piglin::PiglinEntity;
use crate::entity::mob::piglin_ai::PiglinAi;

const MAX_COOLDOWN_BEFORE_RETRYING: i32 = 40;

/// Walks an admiring piglin to the nearest item it wants.
pub struct GoToWantedItemGoal {
    goal_control: Controls,
    piglin: Arc<PiglinEntity>,
    speed: f64,
    last_target_block: Option<BlockPos>,
    remaining_cooldown: i32,
}

impl GoToWantedItemGoal {
    #[must_use]
    pub fn new(piglin: Arc<PiglinEntity>, speed: f64) -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK,
            piglin,
            speed,
            last_target_block: None,
            remaining_cooldown: 0,
        }
    }

    fn wanted_item_in_walking_range(&self) -> Option<Arc<dyn EntityBase>> {
        if !self.piglin.mob_entity.can_pick_up_loot()
            || !PiglinAi::is_not_holding_loved_item_in_off_hand(&self.piglin)
        {
            return None;
        }

        let entity = &self.piglin.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        self.piglin.nearest_wanted_item().filter(|item| {
            let item_pos = item.get_entity().pos.load();
            let max = PiglinAi::MAX_DISTANCE_TO_WALK_TO_ITEM;
            let block = item_pos.to_block_pos();
            item_pos.squared_distance_to_vec(&pos) < max * max
                && world
                    .worldborder
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .contains_block(block.0.x, block.0.z)
        })
    }

    fn go_to_wanted_item(&mut self, mob: &dyn Mob) {
        let Some(target) = self.wanted_item_in_walking_range() else {
            return;
        };

        let mob_entity = mob.get_mob_entity();
        mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity(mob, &target);

        let item_pos = target.get_entity().pos.load();
        let target_block = BlockPos::floored_v(item_pos);
        let mob_block = mob_entity.living_entity.entity.block_pos.load();
        let mut navigator = mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let reached = (target_block.0.x - mob_block.0.x).abs()
            + (target_block.0.y - mob_block.0.y).abs()
            + (target_block.0.z - mob_block.0.z).abs()
            == 0;
        if reached {
            if !navigator.is_idle() {
                navigator.stop();
            }
            self.last_target_block = None;
            return;
        }

        if navigator.is_idle() {
            if self.last_target_block.take().is_some() && navigator.is_stuck() {
                self.remaining_cooldown = rand::random_range(0..MAX_COOLDOWN_BEFORE_RETRYING);
            }
            if self.remaining_cooldown > 0 {
                self.remaining_cooldown -= 1;
                return;
            }
        }

        let moved_far = self.last_target_block.is_none_or(|last| {
            let dx = target_block.0.x - last.0.x;
            let dy = target_block.0.y - last.0.y;
            let dz = target_block.0.z - last.0.z;
            dx * dx + dy * dy + dz * dz > 4
        });
        if navigator.is_idle() || moved_far {
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            navigator.set_progress(NavigatorGoal::new(mob_pos, item_pos, self.speed));
            self.last_target_block = Some(target_block);
        }
    }
}

impl Goal for GoToWantedItemGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.piglin.is_admiring()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.piglin.is_admiring()
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.go_to_wanted_item(mob);
        self.piglin.stop_admiring_if_item_too_far_away();
        self.piglin.stop_admiring_if_tired_of_trying_to_reach_item();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
        self.last_target_block = None;
        self.remaining_cooldown = 0;
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
