use super::{Controls, Goal, GoalReach, to_goal_ticks};
use crate::entity::ai::util::{default_random_pos, land_random_pos};
use crate::entity::{ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub const DEFAULT_INTERVAL: i32 = 120;
pub const PROBABILITY: f32 = 0.001;

pub struct WanderAroundGoal {
    goal_control: Controls,
    speed: f64,
    target: Option<Vector3<f64>>,
    interval: i32,
    force_trigger: bool,
    water_avoiding_probability: Option<f32>,
}

impl WanderAroundGoal {
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
            force_trigger: false,
            water_avoiding_probability: None,
        }
    }

    #[must_use]
    pub const fn water_avoiding(speed: f64) -> Self {
        let mut goal = Self::new(speed);
        goal.water_avoiding_probability = Some(PROBABILITY);
        goal
    }

    /// Run once on the next check, ignoring the interval.
    pub const fn trigger(&mut self) {
        self.force_trigger = true;
    }

    pub const fn set_interval(&mut self, interval: i32) {
        self.interval = interval;
    }

    fn get_position(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        let Some(probability) = self.water_avoiding_probability else {
            return default_random_pos::get_pos(mob, 10, 7);
        };
        if mob.get_entity().is_in_water() {
            land_random_pos::get_pos(mob, 15, 7).or_else(|| default_random_pos::get_pos(mob, 10, 7))
        } else if mob.get_random().random::<f32>() >= probability {
            land_random_pos::get_pos(mob, 10, 7)
        } else {
            default_random_pos::get_pos(mob, 10, 7)
        }
    }
}

impl Goal for WanderAroundGoal {
    fn tick_reach(&self, _mob: &dyn Mob, _running: bool) -> Option<GoalReach> {
        Some(GoalReach {
            speed: self.speed,
            ..GoalReach::default()
        })
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        // Every mob that carries a passenger is controlled by it.
        if mob.get_entity().has_passengers() {
            return false;
        }

        if !self.force_trigger {
            // TODO: also bail out on a long no-action time, which needs the despawn bookkeeping.
            if mob
                .get_random()
                .random_range(0..to_goal_ticks(self.interval))
                != 0
            {
                return false;
            }
        }

        self.target = self.get_position(mob);
        if self.target.is_none() {
            return false;
        }
        self.force_trigger = false;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let idle = mob.is_navigator_idle();
        !idle && !mob.get_entity().has_passengers()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            mob.get_mob_entity()
                .steered()
                .navigator()
                .set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        mob.get_mob_entity().steered().navigator().stop();
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
