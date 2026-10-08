use super::{Controls, Goal, to_goal_ticks};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::{default_random_pos, land_random_pos};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub const DEFAULT_INTERVAL: i32 = 120;
pub const PROBABILITY: f32 = 0.001;

pub struct WaterAvoidingRandomStrollGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    interval: i32,
    probability: f32,
    force_trigger: bool,
}

impl WaterAvoidingRandomStrollGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self::with_interval(speed, DEFAULT_INTERVAL)
    }

    #[must_use]
    pub const fn with_interval(speed: f64, interval: i32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            target: None,
            interval,
            probability: PROBABILITY,
            force_trigger: false,
        }
    }

    pub const fn trigger(&mut self) {
        self.force_trigger = true;
    }

    pub const fn set_interval(&mut self, interval: i32) {
        self.interval = interval;
    }

    fn get_position(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        if mob.get_entity().is_in_water() {
            land_random_pos::get_pos(mob, 15, 7).or_else(|| default_random_pos::get_pos(mob, 10, 7))
        } else if mob.get_random().random::<f32>() >= self.probability {
            land_random_pos::get_pos(mob, 10, 7).or_else(|| default_random_pos::get_pos(mob, 10, 7))
        } else {
            default_random_pos::get_pos(mob, 10, 7)
        }
    }
}

impl Goal for WaterAvoidingRandomStrollGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_entity().has_passengers() {
            return false;
        }

        if mob
            .get_mob_entity()
            .get_target()
            .is_some_and(|t| t.get_entity().is_alive())
        {
            return false;
        }

        let interval = self.interval;

        if !self.force_trigger
            && mob
                .get_random()
                .random_range(0..to_goal_ticks(interval))
                != 0
        {
            return false;
        }

        self.target = self.get_position(mob);
        if self.target.is_none() {
            return false;
        }
        self.force_trigger = false;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle() && !mob.get_entity().has_passengers()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let mut nav = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            nav.set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
