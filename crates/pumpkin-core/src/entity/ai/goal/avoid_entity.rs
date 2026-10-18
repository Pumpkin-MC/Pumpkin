use std::sync::Arc;

use super::{Controls, Goal};
use crate::entity::ai::goal::revenge::MobFilter;
use crate::entity::ai::target_match::{SearchArea, TargetMatch};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::ai::util::default_random_pos;
use crate::entity::predicate::EntityPredicate;
use crate::entity::{EntityBase, ai::pathfinder::NavigatorGoal, mob::Mob};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

const FAST_DISTANCE_SQ: f64 = 49.0;
const HORIZONTAL_RANGE: i32 = 16;
const VERTICAL_RANGE: i32 = 7;
/// Vanilla only looks this far above and below for threats.
const THREAT_SEARCH_HEIGHT: f64 = 3.0;

/// A check on the threat that was found.
pub type ThreatFilter = fn(&dyn EntityBase) -> bool;

/// Vanilla's `predicateOnAvoidEntity`, checked on every candidate: fleeing mob, candidate.
pub type ThreatSelector = fn(&dyn Mob, &dyn EntityBase) -> bool;

/// The default selector: no creative or spectator players.
pub fn not_creative_or_spectator(_mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    EntityPredicate::ExceptCreativeOrSpectator.test(candidate.get_entity())
}

/// Fish and pandas: creative players count too.
pub fn not_spectator(_mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    !candidate.get_entity().is_spectator()
}

/// Threat types, each with the distance it starts the flee at.
pub type ThreatDistances = &'static [(&'static EntityType, f64)];

pub struct AvoidEntityGoal {
    goal_control: Controls,
    flee_from: TargetMatch,
    flee_distance: f64,
    threat_predicate: TargetPredicate,
    slow_speed: f64,
    fast_speed: f64,
    selector: ThreatSelector,
    target: Option<Arc<dyn EntityBase>>,
    flee_pos: Option<Vector3<f64>>,
    when: Option<MobFilter>,
    keep_when: bool,
    confirm: Option<ThreatFilter>,
    drop_target: bool,
    per_type: Option<ThreatDistances>,
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
            // Vanilla `forCombat`: line of sight, can attack, not allied, visibility-scaled range.
            threat_predicate: TargetPredicate::create_attackable()
                .set_base_max_distance(flee_distance),
            slow_speed,
            fast_speed,
            selector: not_creative_or_spectator,
            target: None,
            flee_pos: None,
            when: None,
            keep_when: false,
            confirm: None,
            drop_target: false,
            per_type: None,
        }
    }

    /// Vanilla brain sensors (`VillagerHostilesSensor`): the nearest visible threat
    /// within its own type's distance, seen like any entity, no height limit.
    #[must_use]
    pub fn nearest_of(threats: ThreatDistances, slow_speed: f64, fast_speed: f64) -> Self {
        let reach = threats
            .iter()
            .fold(0.0, |max, &(_, dist)| f64::max(max, dist));
        Self {
            threat_predicate: TargetPredicate::create_non_attackable(),
            per_type: Some(threats),
            ..Self::new(TargetMatch::Any, reach, slow_speed, fast_speed)
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

    /// Replaces the default creative and spectator check on every candidate.
    #[must_use]
    pub const fn matching(mut self, selector: ThreatSelector) -> Self {
        self.selector = selector;
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
        let living = &mob.get_mob_entity().living_entity;
        let entity = &living.entity;
        let pos = entity.pos.load();
        let world = entity.world.load();
        let height = if self.per_type.is_some() {
            self.flee_distance
        } else {
            THREAT_SEARCH_HEIGHT
        };
        let area = SearchArea::from_feet(entity, self.flee_distance, height);
        // Sensors see as far as the follow range; the per-type distance cuts it down after.
        let sensor_predicate;
        let predicate = if self.per_type.is_some() {
            sensor_predicate = self
                .threat_predicate
                .copy()
                .set_base_max_distance(living.get_attribute_value(&Attributes::FOLLOW_RANGE));
            &sensor_predicate
        } else {
            &self.threat_predicate
        };

        self.flee_from.find_nearest(&world, &area, |candidate| {
            let candidate_entity = candidate.get_entity();
            self.per_type.is_none_or(|threats| {
                threats.iter().any(|&(threat, dist)| {
                    threat == candidate_entity.entity_type
                        && candidate_entity.pos.load().squared_distance_to_vec(&pos) <= dist * dist
                })
            }) && (self.selector)(mob, candidate)
                && predicate.test(&world, Some(mob), candidate)
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
            mob.set_mob_target(None, TargetReason::Fleeing);
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
            mob.set_mob_target(None, TargetReason::Fleeing);
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
