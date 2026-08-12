use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use super::{Controls, Goal};
use crate::entity::EntityBase;
use crate::entity::ai::goal::GoalFuture;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;

pub struct RevengeGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    last_attacked_time: i32,
    target_predicate: TargetPredicate,
    alert_others: bool,
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
            alert_others: false,
        }
    }

    #[must_use]
    pub fn with_alert_others(check_visibility: bool) -> Self {
        let mut goal = Self::new(check_visibility);
        goal.alert_others = true;
        goal
    }
}

impl Goal for RevengeGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
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
                .await
            {
                return false;
            }

            self.target = Some(attacker);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.target.clone()).await;

            let mob_entity = mob.get_mob_entity();
            self.last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
            self.track_target_goal.max_time_without_visibility = 300;

            if self.alert_others {
                let world = mob_entity.living_entity.entity.world.load();
                let target = self.target.clone();
                let search = mob_entity.living_entity.entity.bounding_box.load().expand(
                    mob_entity
                        .living_entity
                        .get_attribute_value(&pumpkin_data::attributes::Attributes::FOLLOW_RANGE),
                    10.0,
                    mob_entity
                        .living_entity
                        .get_attribute_value(&pumpkin_data::attributes::Attributes::FOLLOW_RANGE),
                );
                let mut nearby = Vec::new();
                world.extend_entities_in_box_where(&mut nearby, usize::MAX, search, |entity| {
                    entity.get_entity().entity_type == mob.get_entity().entity_type
                });
                let owner = mob.get_owner_uuid();
                for other in nearby {
                    let Some(other_mob) = other.get_mob() else {
                        continue;
                    };
                    if other.get_entity().entity_uuid == mob.get_entity().entity_uuid
                        || other_mob.get_mob_entity().get_target().await.is_some()
                        || other_mob.get_owner_uuid() != owner
                    {
                        continue;
                    }
                    let Some(target) = target.as_ref() else {
                        continue;
                    };
                    if world.are_allied(other.as_ref(), target.as_ref()).await {
                        continue;
                    }
                    other_mob.set_mob_target(Some(target.clone())).await;
                }
            }

            self.track_target_goal.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            self.target = None;
            self.track_target_goal.stop(mob).await;
        })
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}
