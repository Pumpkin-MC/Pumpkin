use std::sync::Arc;

use super::{Controls, Goal};
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::ai::util::default_random_pos;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};
use pumpkin_util::math::vector3::Vector3;

const FAST_DISTANCE_SQ: f64 = 49.0;
const HORIZONTAL_RANGE: i32 = 16;
const VERTICAL_RANGE: i32 = 7;

/// A check on the threat that was found.
pub type ThreatFilter = fn(&dyn EntityBase) -> bool;

pub struct AvoidEntityGoal {
    goal_control: Controls,
    flee_from: TargetMatch,
    flee_distance: f64,
    slow_speed: f64,
    fast_speed: f64,
    target: Option<Arc<dyn EntityBase>>,
    flee_pos: Option<Vector3<f64>>,
    when: Option<MobFilter>,
    keep_when: bool,
    confirm: Option<ThreatFilter>,
    drop_target: bool,
}

impl AvoidEntityGoal {
    #[must_use]
    pub fn new(
        flee_from: impl Into<TargetMatch>,
        flee_distance: f64,
        slow_speed: f64,
        fast_speed: f64,
    ) -> Self {
        Self {
            goal_control: Controls::MOVE,
            flee_from: flee_from.into(),
            flee_distance,
            slow_speed,
            fast_speed,
            target: None,
            flee_pos: None,
            when: None,
            keep_when: false,
            confirm: None,
            drop_target: false,
        }
    }

    /// Only flees while the mob passes `filter`
    #[must_use]
    pub const fn when(mut self, filter: MobFilter) -> Self {
        self.when = Some(filter);
        self
    }

    /// Like [`Self::when`], also checked while fleeing. e.g. a cat tamed mid-flight stops.
    #[must_use]
    pub const fn as_long_as(mut self, filter: MobFilter) -> Self {
        self.when = Some(filter);
        self.keep_when = true;
        self
    }

    /// The nearest threat must also pass `filter`, else no flee
    #[must_use]
    pub const fn confirmed_by(mut self, filter: ThreatFilter) -> Self {
        self.confirm = Some(filter);
        self
    }

    /// Clears the mob's target while fleeing.
    #[must_use]
    pub const fn dropping_target(mut self) -> Self {
        self.drop_target = true;
        self
    }

    fn find_threat(&self, mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let entity = &mob.get_mob_entity().living_entity.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();

        self.flee_from
            .find_nearest(&world, pos, self.flee_distance, |entity| {
                EntityPredicate::ExceptCreativeOrSpectator.test(entity.get_entity())
            })
    }
}

impl Goal for AvoidEntityGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.when.is_some_and(|when| !when(mob)) {
            return false;
        }
        let Some(target) = self.find_threat(mob) else {
            return false;
        };
        if self
            .confirm
            .is_some_and(|confirm| !confirm(target.as_ref()))
        {
            return false;
        }

        let threat_pos = target.get_entity().pos.load();
        let Some(flee_pos) =
            default_random_pos::get_pos_away(mob, HORIZONTAL_RANGE, VERTICAL_RANGE, threat_pos)
        else {
            return false;
        };

        // Give up when the escape route does not gain any distance.
        let mob_pos = mob.get_entity().pos.load();
        if threat_pos.squared_distance_to_vec(&flee_pos)
            < threat_pos.squared_distance_to_vec(&mob_pos)
        {
            return false;
        }

        self.target = Some(target);
        self.flee_pos = Some(flee_pos);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.keep_when && self.when.is_some_and(|when| !when(mob)) {
            return false;
        }
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if self.drop_target {
            mob.set_mob_target(None);
        }
        if let Some(flee_pos) = self.flee_pos {
            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_progress(NavigatorGoal::new(mob_pos, flee_pos, self.slow_speed));
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if self.drop_target {
            mob.set_mob_target(None);
        }
        if let Some(target) = &self.target {
            let mob_pos = mob.get_mob_entity().living_entity.entity.pos.load();
            let threat_pos = target.get_entity().pos.load();
            let dist_sq = mob_pos.squared_distance_to_vec(&threat_pos);
            let speed = if dist_sq < FAST_DISTANCE_SQ {
                self.fast_speed
            } else {
                self.slow_speed
            };
            let mut navigator = mob
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            navigator.set_speed(speed);
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target = None;
        self.flee_pos = None;
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}
