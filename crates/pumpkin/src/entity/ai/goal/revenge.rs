use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::EntityBase;

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;

/// A function pointer also covers whole hierarchies, like every raider.
pub type EntityTypeFilter = fn(&'static EntityType) -> bool;

const ALERT_RANGE_Y: f64 = 10.0;

pub struct RevengeGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_attacked_time: i32,
    target_predicate: TargetPredicate,
    ignore_damage_from: Option<EntityTypeFilter>,
    alert_others: bool,
    ignore_alert: Option<EntityTypeFilter>,
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

    /// Wake up nearby mobs of the same kind.
    fn alert_others(&self, mob: &dyn Mob, attacker: &Arc<dyn EntityBase>) {
        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let within = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);

        let world = entity.world.load();
        let search_box = entity
            .bounding_box
            .load()
            .expand(within, ALERT_RANGE_Y, within);

        for other in world.get_entities_at_box(&search_box) {
            let other_entity = other.get_entity();
            if other_entity.entity_id == entity.entity_id
                || other_entity.entity_type != entity.entity_type
            {
                continue;
            }
            let Some(other_mob) = other.get_mob() else {
                continue;
            };
            if other_mob.get_mob_entity().get_target().is_some() {
                continue;
            }
            if mob.is_tamed() && mob.get_owner_uuid() != other_mob.get_owner_uuid() {
                continue;
            }
            if other.is_allied_to(attacker.as_ref()) {
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
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.set_mob_target(self.target.clone());

        let mob_entity = mob.get_mob_entity();
        self.track_target_goal.set_target_mob(self.target.clone());
        self.last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
        self.track_target_goal.max_time_without_visibility = 300;

        if self.alert_others
            && let Some(attacker) = self.target.clone()
        {
            self.alert_others(mob, &attacker);
        }

        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
