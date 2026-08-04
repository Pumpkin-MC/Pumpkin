use std::sync::Arc;

use pumpkin_data::attributes::Attributes;

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::vindicator::VindicatorEntity;
use crate::entity::{EntityBase, mob::MobEntity};

/// Vanilla: `Vindicator.VindicatorJohnnyAttackGoal extends NearestAttackableTargetGoal<LivingEntity>`.
///
/// Unlike every other `NearestAttackableTargetGoal` in the codebase (which target one concrete
/// `EntityType`), a "Johnny" vindicator targets *any* living entity -- gated on `is_johnny`, the
/// one-way latch set when the vindicator is named "Johnny".
pub struct JohnnyAttackGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    target_predicate: TargetPredicate,
}

impl JohnnyAttackGoal {
    #[must_use]
    pub fn new(mob: &MobEntity) -> Self {
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        Self {
            track_target_goal: TrackTargetGoal::new(true, true),
            target: None,
            target_predicate,
        }
    }

    async fn find_closest_living_entity(&mut self, mob_entity: &MobEntity) {
        let follow_range = mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        self.target_predicate.base_max_distance = follow_range;

        let world = mob_entity.living_entity.entity.world.load();
        let mut search_pos = mob_entity.living_entity.entity.pos.load();
        search_pos.y += mob_entity
            .living_entity
            .entity
            .entity_dimension
            .load()
            .eye_height as f64;
        let self_id = mob_entity.living_entity.entity.entity_id;

        let mut candidates: Vec<Arc<dyn EntityBase>> = world
            .get_nearby_entities(search_pos, follow_range)
            .into_values()
            .filter(|entity| {
                entity.get_entity().entity_id != self_id && entity.get_living_entity().is_some()
            })
            .collect();
        candidates.sort_by(|a, b| {
            a.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&search_pos)
                .partial_cmp(
                    &b.get_entity()
                        .pos
                        .load()
                        .squared_distance_to_vec(&search_pos),
                )
                .unwrap()
        });

        self.target = None;
        for entity in candidates {
            if let Some(living) = entity.get_living_entity()
                && self
                    .target_predicate
                    .test(&world, Some(&mob_entity.living_entity), living)
                    .await
            {
                self.target = Some(entity);
                break;
            }
        }
    }
}

impl Goal for JohnnyAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if !mob
                .cast_any()
                .downcast_ref::<VindicatorEntity>()
                .is_some_and(VindicatorEntity::is_johnny)
            {
                return false;
            }
            self.find_closest_living_entity(mob.get_mob_entity()).await;
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async { self.track_target_goal.should_continue(mob).await })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async {
            mob.set_mob_target(self.target.clone()).await;
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
