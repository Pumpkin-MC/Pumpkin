use std::sync::{Arc, Weak};

use crate::entity::EntityBase;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::mob::{Mob, RangedAttackMob};

/// Generic ranged attack goal for mobs implementing [`RangedAttackMob`].
///
/// Mirrors vanilla `RangedAttackGoal`. The mob maintains distance from its target,
/// points towards it, and fires ranged attacks periodically with calculated power.
pub struct RangedAttackGoal {
    mob: Weak<dyn RangedAttackMob>,
    speed_modifier: f64,
    attack_interval_min: i32,
    attack_interval_max: i32,
    attack_radius: f32,
    attack_radius_sqr: f64,
    see_time: i32,
    attack_time: i32,
    target: Option<Arc<dyn EntityBase>>,
}

impl RangedAttackGoal {
    #[must_use]
    pub const fn new(
        mob: Weak<dyn RangedAttackMob>,
        speed_modifier: f64,
        attack_interval: i32,
        attack_radius: f32,
    ) -> Self {
        Self::new_with_range(
            mob,
            speed_modifier,
            attack_interval,
            attack_interval,
            attack_radius,
        )
    }

    #[must_use]
    pub const fn new_with_range(
        mob: Weak<dyn RangedAttackMob>,
        speed_modifier: f64,
        attack_interval_min: i32,
        attack_interval_max: i32,
        attack_radius: f32,
    ) -> Self {
        Self {
            mob,
            speed_modifier,
            attack_interval_min,
            attack_interval_max,
            attack_radius,
            attack_radius_sqr: (attack_radius * attack_radius) as f64,
            see_time: 0,
            attack_time: -1,
            target: None,
        }
    }
}

impl Goal for RangedAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().get_target().clone();
        if let Some(target) = target
            && target.get_entity().is_alive()
        {
            self.target = Some(target);
            true
        } else {
            false
        }
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        // Vanilla re-runs `canUse`, which switches to the mob's current target; without one
        // the goal keeps walking to the old target while it lives and the path is not done.
        self.can_start(mob)
            || (self
                .target
                .as_ref()
                .is_some_and(|target| target.get_entity().is_alive())
                && !mob.get_mob_entity().steered().navigator().is_done())
    }

    // Vanilla leaves the navigation running.
    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
        self.see_time = 0;
        self.attack_time = -1;
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(target) = self.target.clone() else {
            return;
        };

        if self.mob.upgrade().is_none() {
            return;
        }

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let target_dist_sq = mob_pos.squared_distance_to_vec(&target_pos);

        let has_line_of_sight = mob.has_line_of_sight(target.get_entity());
        if has_line_of_sight {
            self.see_time += 1;
        } else {
            self.see_time = 0;
        }

        {
            let steered = mob.get_mob_entity().steered();
            let mut navigator = steered.navigator();
            if target_dist_sq <= self.attack_radius_sqr && self.see_time >= 5 {
                navigator.stop();
            } else {
                navigator.set_progress(NavigatorGoal {
                    current_progress: mob_pos,
                    destination: target_pos,
                    speed: self.speed_modifier,
                });
            }
        }

        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 30.0, 30.0);

        self.attack_time -= 1;
        if self.attack_time == 0 {
            if !has_line_of_sight {
                return;
            }

            let dist = (target_dist_sq.sqrt() as f32) / self.attack_radius;
            let power = dist.clamp(0.1, 1.0);
            if let Some(ranged) = self.mob.upgrade() {
                ranged.perform_ranged_attack(&target, power);
            }

            let min = self.attack_interval_min as f32;
            let max = self.attack_interval_max as f32;
            self.attack_time = (dist.mul_add(max - min, min)).floor() as i32;
        } else if self.attack_time < 0 {
            // `Mth.lerp` in double precision, unlike the float interval above.
            let ratio = target_dist_sq.sqrt() / f64::from(self.attack_radius);
            let min = f64::from(self.attack_interval_min);
            let max = f64::from(self.attack_interval_max);
            self.attack_time = ratio.mul_add(max - min, min).floor() as i32;
        }
    }
}
