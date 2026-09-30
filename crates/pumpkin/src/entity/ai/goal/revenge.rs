use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::EntityBase;

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::ai::util::goal_utils;
use crate::entity::mob::Mob;
use pumpkin_data::entity::EntityType;

/// A function pointer also covers whole hierarchies, like every raider.
pub type EntityTypeFilter = fn(&'static EntityType) -> bool;

/// A check on a single mob.
pub type MobFilter = fn(&dyn Mob) -> bool;

pub struct RevengeGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_attacked_time: i32,
    target_predicate: TargetPredicate,
    ignore_damage_from: Option<EntityTypeFilter>,
    alert_others: bool,
    ignore_alert: Option<EntityTypeFilter>,
    alert_only: Option<MobFilter>,
    alarm_when: Option<MobFilter>,
    continue_while: Option<MobFilter>,
    alert_in_sight: bool,
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
            ignore_damage_from: None,
            alert_others: false,
            ignore_alert: None,
            alert_only: None,
            alarm_when: None,
            continue_while: None,
            alert_in_sight: false,
        }
    }

    #[must_use]
    pub const fn ignoring(mut self, filter: EntityTypeFilter) -> Self {
        self.ignore_damage_from = Some(filter);
        self
    }

    #[must_use]
    pub const fn alerting_others(mut self) -> Self {
        self.alert_others = true;
        self
    }

    #[must_use]
    pub const fn alerting_others_except(mut self, filter: EntityTypeFilter) -> Self {
        self.alert_others = true;
        self.ignore_alert = Some(filter);
        self
    }

    /// Only mobs passing `filter` can be alerted.
    #[must_use]
    pub const fn alerting_only(mut self, filter: MobFilter) -> Self {
        self.alert_only = Some(filter);
        self
    }

    /// While `filter` holds for the mob, being hurt only raises the alarm: the mob alerts
    /// others, then drops the grudge itself.
    #[must_use]
    pub const fn raising_alarm_when(mut self, filter: MobFilter) -> Self {
        self.alarm_when = Some(filter);
        self
    }

    /// Extra condition for keeping the target, e.g. a bee that is still angry.
    #[must_use]
    pub const fn continuing_while(mut self, filter: MobFilter) -> Self {
        self.continue_while = Some(filter);
        self
    }

    /// Others are only alerted while the mob can see the attacker.
    #[must_use]
    pub const fn alerting_in_sight(mut self) -> Self {
        self.alert_in_sight = true;
        self
    }

    /// Wake up nearby mobs of the same kind.
    fn alert_others(&self, mob: &dyn Mob, attacker: &Arc<dyn EntityBase>) {
        if self.alert_in_sight && !mob.has_line_of_sight(attacker.get_entity()) {
            return;
        }
        for other in goal_utils::nearby_same_type(mob) {
            let other_entity = other.get_entity();
            let Some(other_mob) = other.get_mob() else {
                continue;
            };
            if other_mob.get_mob_entity().get_target().is_some() {
                continue;
            }
            if mob.as_tamable().is_some() && mob.get_owner_uuid() != other_mob.get_owner_uuid() {
                continue;
            }
            if other.is_allied_to(attacker.as_ref()) {
                continue;
            }
            if self.alert_only.is_some_and(|filter| !filter(other_mob)) {
                continue;
            }
            if self
                .ignore_alert
                .is_some_and(|filter| filter(other_entity.entity_type))
            {
                continue;
            }
            other_mob.set_mob_target(Some(attacker.clone()));
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

        // Universal anger is handled by its own goal instead.
        if attacker.get_entity().entity_type == &EntityType::PLAYER
            && world.level_info.load().game_rules.universal_anger
        {
            return false;
        }

        if self
            .ignore_damage_from
            .is_some_and(|filter| filter(attacker.get_entity().entity_type))
        {
            return false;
        }

        if !self
            .track_target_goal
            .can_track(mob, Some(attacker.as_ref()), &self.target_predicate)
        {
            return false;
        }

        self.target = Some(attacker);
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.continue_while.is_none_or(|filter| filter(mob))
            && self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.target.clone());

        let mob_entity = mob.get_mob_entity();
        self.track_target_goal.set_target_mob(self.target.clone());
        self.last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
        self.track_target_goal.max_time_without_visibility = 300;

        let alarm = self.alarm_when.is_some_and(|filter| filter(mob));
        if (self.alert_others || alarm)
            && let Some(attacker) = self.target.clone()
        {
            self.alert_others(mob, &attacker);
        }

        self.track_target_goal.start(mob);
        if alarm {
            self.track_target_goal.stop(mob);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
