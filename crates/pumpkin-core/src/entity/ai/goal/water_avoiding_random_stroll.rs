use super::{Controls, Goal, to_goal_ticks, wander_around::DEFAULT_INTERVAL};
use crate::entity::ai::pathfinder::NavigatorGoal;
use crate::entity::ai::util::{default_random_pos, land_random_pos};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

/// Vanilla `WaterAvoidingRandomStrollGoal.PROBABILITY`, the two-argument constructor's default.
pub const PROBABILITY: f32 = 0.001;

/// Vanilla `WaterAvoidingRandomStrollGoal`: a `RandomStrollGoal` whose destination comes from
/// `LandRandomPos` (which refuses water), searching 15 blocks while the mob is in water and 10
/// otherwise.
pub struct WaterAvoidingRandomStrollGoal {
    goal_control: Controls,
    speed: f64,
    probability: f32,
    target: Option<Vector3<f64>>,
    interval: i32,
}

impl WaterAvoidingRandomStrollGoal {
    /// Creates the goal with the vanilla 120-tick stroll interval (`wander_around`'s default)
    /// and the vanilla default `probability`.
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self::with_probability(speed, PROBABILITY)
    }

    /// Like [`Self::new`], with a custom chance of heading for a plain (possibly wet) spot on
    /// land; the feline goals pass vanilla's `1.0000001E-5`.
    #[must_use]
    pub const fn with_probability(speed: f64, probability: f32) -> Self {
        Self {
            goal_control: Controls::MOVE,
            speed,
            probability,
            target: None,
            interval: DEFAULT_INTERVAL,
        }
    }

    /// Vanilla `WaterAvoidingRandomStrollGoal.getPosition`: in water, look for land within 15
    /// blocks and fall back to the plain lookup when none is found; on land, roll against
    /// `probability` and normally take the water-refusing lookup. Both plain fallbacks are
    /// vanilla's `super.getPosition()`, `DefaultRandomPos` at 10 by 7.
    fn get_position(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        if mob.get_entity().is_in_water() {
            land_random_pos::get_pos(mob, 15, 7).or_else(|| default_random_pos::get_pos(mob, 10, 7))
        } else if mob.get_random().random_range(0.0f32..1.0) >= self.probability {
            land_random_pos::get_pos(mob, 10, 7)
        } else {
            default_random_pos::get_pos(mob, 10, 7)
        }
    }
}

impl Goal for WaterAvoidingRandomStrollGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        // Every mob that carries a passenger is controlled by it.
        if mob.get_entity().has_passengers() {
            return false;
        }

        if mob
            .get_random()
            .random_range(0..to_goal_ticks(self.interval))
            != 0
        {
            return false;
        }

        self.target = self.get_position(mob);
        self.target.is_some()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let idle = mob.is_navigator_idle();
        !idle && !mob.get_entity().has_passengers()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(target) = self.target {
            let pos = mob.get_mob_entity().living_entity.entity.pos.load();
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(NavigatorGoal::new(pos, target, self.speed));
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
