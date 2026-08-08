use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::attributes::Attributes;
use rand::RngExt;

use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::goal::{Controls, Goal, GoalFuture, to_goal_ticks};
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::mob::Mob;
use crate::entity::mob::witch::WitchEntity;
use crate::entity::{EntityBase, mob::MobEntity};

const RANDOM_INTERVAL: i32 = 10;

/// Vanilla: `NearestAttackableWitchTargetGoal<Player>` as wired by `Witch.registerGoals`.
///
/// Same as `NearestAttackableTargetGoal<Player>` except gated by an externally-driven
/// `can_attack` flag (`Witch::mob_tick` sets it from the heal-goal's cooldown, so the witch only
/// starts attacking players while healing is on cooldown).
pub struct NearestAttackableWitchTargetGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    target_predicate: TargetPredicate,
}

impl NearestAttackableWitchTargetGoal {
    #[must_use]
    pub fn new(mob: &MobEntity) -> Self {
        let mut target_predicate = TargetPredicate::create_attackable();
        target_predicate.base_max_distance = mob
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        Self {
            track_target_goal: TrackTargetGoal::with_default(true),
            target: None,
            target_predicate,
        }
    }

    async fn find_closest_player(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
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

        let mut candidates = world.get_nearby_players(search_pos, follow_range);
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
        for player in candidates {
            if !TrackTargetGoal::is_allied(mob, player.as_ref()).await
                && mob.can_attack(player.get_entity())
                && self
                    .target_predicate
                    .test(
                        &world,
                        Some(&mob_entity.living_entity),
                        &player.living_entity,
                    )
                    .await
            {
                self.target = Some(player as Arc<dyn EntityBase>);
                break;
            }
        }
    }
}

impl Goal for NearestAttackableWitchTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(witch) = mob.cast_any().downcast_ref::<WitchEntity>() else {
                return false;
            };
            if !witch.can_attack_players.load(Relaxed) {
                return false;
            }
            if rand::rng().random_range(0..to_goal_ticks(RANDOM_INTERVAL)) != 0 {
                return false;
            }
            self.find_closest_player(mob).await;
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
