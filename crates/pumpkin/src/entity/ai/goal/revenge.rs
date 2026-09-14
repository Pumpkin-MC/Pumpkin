use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::EntityBase;

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;

pub struct RevengeGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_attacked_time: i32,
    target_predicate: TargetPredicate,
}

impl RevengeGoal {
    #[must_use]
    pub fn new(check_visibility: bool) -> Self {
        let target_predicate = TargetPredicate::create_attackable()
            .ignore_visibility()
            .ignore_distance_scaling_factor();
        Self {
            track_target_goal: TrackTargetGoal::with_default(check_visibility),
            target: None,
            last_attacked_time: 0,
            target_predicate,
        }
    }
}

impl Goal for RevengeGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;

        let attacked_time = living.last_attacked_time.load(Relaxed);
        if attacked_time == self.last_attacked_time {
            return false;
        }

        let attacker_id = living.last_attacker_id.load(Relaxed);
        if attacker_id == 0 {
            return false;
        }

        let world = living.entity.world.load();
        let Some(attacker) = world.get_entity_by_id(attacker_id) else {
            return false;
        };

        let Some(attacker_living) = attacker.get_living_entity() else {
            return false;
        };

        if !self
            .target_predicate
            .test(&world, Some(&mob_entity.living_entity), attacker_living)
        {
            return false;
        }

        self.target = Some(attacker);
        true
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.target.clone());

        let mob_entity = mob.get_mob_entity();
        self.last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
        self.track_target_goal.max_time_without_visibility = 300;

        self.track_target_goal.start(mob);
        // TODO: group revenge — call nearby mobs of same type to help
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
