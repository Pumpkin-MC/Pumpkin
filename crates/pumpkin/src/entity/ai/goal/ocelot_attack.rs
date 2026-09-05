use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::Mob;
use std::sync::Arc;

pub struct OcelotAttackGoal {
    goal_control: Controls,
    target: Option<Arc<dyn EntityBase>>,
    attack_time: i32,
}

impl Default for OcelotAttackGoal {
    fn default() -> Self {
        Self::new()
    }
}

impl OcelotAttackGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            goal_control: Controls::MOVE.union(Controls::LOOK),
            target: None,
            attack_time: 0,
        }
    }
}

impl Goal for OcelotAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target();
        let Some(target) = target else {
            return false;
        };

        if !target.get_entity().is_alive() {
            return false;
        }

        self.target = Some(target);
        true
    }

    fn should_continue(&self, mob: &dyn Mob) -> bool {
        let Some(target) = &self.target else {
            return false;
        };

        if !target.get_entity().is_alive() {
            return false;
        }

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

        if dist_sq > 225.0 {
            return false;
        }

        let is_idle = mob
            .get_mob_entity()
            .navigator
            .try_lock()
            .is_ok_and(|nav| nav.is_idle());

        !is_idle || mob.get_mob_entity().get_target().is_some()
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        let mut navigator = mob
            .get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.stop();
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = &self.target else {
            return;
        };

        let mob_entity = mob.get_mob_entity();
        mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(target, 30.0, 30.0);

        let bb_width = mob.get_entity().entity_dimension.load().width;
        let melee_radius_sq = (bb_width * 2.0) * (bb_width * 2.0);
        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

        let speed_modifier = if dist_sq > f64::from(melee_radius_sq) && dist_sq < 16.0 {
            1.33
        } else if dist_sq < 225.0 {
            0.6
        } else {
            0.8
        };

        let mut navigator = mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_progress(NavigatorGoal::new(mob_pos, target_pos, speed_modifier));

        self.attack_time = (self.attack_time - 1).max(0);
        if dist_sq <= f64::from(melee_radius_sq) && self.attack_time <= 0 {
            self.attack_time = 20;
            mob.get_mob_entity().living_entity.swing_hand();
            mob.get_mob_entity()
                .try_attack(mob.get_entity(), target.as_ref());
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
