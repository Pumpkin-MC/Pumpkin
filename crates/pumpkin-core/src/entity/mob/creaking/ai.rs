use std::sync::Arc;

use pumpkin_data::environment_attribute::Activity;

use super::CreakingEntity;
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainTick,
    activity::ActivityData,
    behavior::{
        DoNothing, LookAtTargetSink, MoveToTargetSink, Swim, Timed, melee_attack, random_stroll,
        run_one, set_entity_look_target_sometimes,
        set_walk_target_from_attack_target_if_target_out_of_reach,
        set_walk_target_from_look_target, start_attacking, stop_attacking_if_target_invalid,
    },
    memory::{MemoryStatus, PackedMemories, types},
    sensing::SensorType,
};
use crate::entity::mob::Mob;
use crate::entity::player::Player;
use crate::plugin::api::events::entity::entity_target::TargetReason;

const ATTACK_INTERVAL: i32 = 40;
const SPEED_MULTIPLIER_WHEN_IDLING: f32 = 0.3;

pub(super) fn make_brain(creaking: &CreakingEntity, packed: &PackedMemories) -> Brain {
    BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::NearestLivingEntities,
            SensorType::NearestPlayers,
        ],
        activities,
    }
    .make_brain(creaking, packed)
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::with_condition(0.8, can_move))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
                Box::new(Timed::new(MoveToTargetSink::default())),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            10,
            vec![
                Box::new(start_attacking::create_with_reason(
                    |tick| {
                        tick.mob
                            .cast_any()
                            .downcast_ref::<CreakingEntity>()
                            .is_some_and(CreakingEntity::is_active)
                    },
                    |tick| {
                        let target: Arc<dyn EntityBase> = tick
                            .state
                            .get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYER)?
                            .clone();
                        Some(target)
                    },
                    |_| TargetReason::ClosestPlayer,
                )),
                Box::new(set_entity_look_target_sometimes::create(8.0, 30, 60)),
                Box::new(run_one(vec![
                    (
                        Box::new(random_stroll::stroll(SPEED_MULTIPLIER_WHEN_IDLING)),
                        2,
                    ),
                    (
                        Box::new(set_walk_target_from_look_target::create(
                            SPEED_MULTIPLIER_WHEN_IDLING,
                            3,
                        )),
                        2,
                    ),
                    (Box::new(DoNothing::new(30, 60)), 1),
                ])),
            ],
        ),
        ActivityData::with_priority_start_and_conditions(
            Activity::Fight,
            10,
            vec![
                Box::new(
                    set_walk_target_from_attack_target_if_target_out_of_reach::create(|_| 1.0),
                ),
                Box::new(melee_attack::create_with(can_move, ATTACK_INTERVAL)),
                Box::new(stop_attacking_if_target_invalid::create_with_target_events(
                    is_target_unreachable,
                    |_, _| {},
                )),
            ],
            vec![(types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent)],
        ),
    ]
}

fn can_move(mob: &dyn Mob) -> bool {
    mob.cast_any()
        .downcast_ref::<CreakingEntity>()
        .is_some_and(CreakingEntity::can_move)
}

fn is_target_unreachable(tick: &BrainTick<'_>, target: &Arc<dyn EntityBase>) -> bool {
    !tick
        .state
        .get(types::NEAREST_VISIBLE_ATTACKABLE_PLAYERS)
        .is_some_and(|players| {
            players
                .iter()
                .any(|player| player.entity_id() == target.get_entity().entity_id)
        })
}

pub(super) fn nearest_players(creaking: &CreakingEntity) -> Vec<Arc<Player>> {
    creaking.mob_entity.with_brain(creaking, |brain| {
        brain
            .get(types::NEAREST_PLAYERS)
            .cloned()
            .unwrap_or_default()
    })
}

pub(super) fn tick(creaking: &CreakingEntity) {
    let mob = &creaking.mob_entity;
    mob.tick_brain(creaking);
    if creaking.get_entity().is_removed() {
        return;
    }
    mob.with_brain(creaking, |brain| {
        if creaking.can_move() {
            brain.set_active_activity_to_first_valid(&[Activity::Fight, Activity::Idle]);
        } else {
            brain.use_default_activity();
        }
    });
}

pub(super) fn tick_links(creaking: &CreakingEntity) -> Vec<i32> {
    let mut links = Vec::new();
    creaking.mob_entity.with_brain(creaking, |brain| {
        if let Some(players) = brain.get(types::NEAREST_PLAYERS) {
            links.extend(players.iter().map(|player| player.entity_id()));
        }
        // Nearby living candidates are range-checked by the look behavior. Its retained
        // target, walk/attack targets and the gaze sensor snapshot can outlive that range.
        let look_target = brain
            .get(types::LOOK_TARGET)
            .and_then(|look| look.tracked_entity());
        let walk_target = brain
            .get(types::WALK_TARGET)
            .and_then(|walk| walk.target().tracked_entity());
        for target in [brain.get(types::ATTACK_TARGET), look_target, walk_target]
            .into_iter()
            .flatten()
        {
            links.push(target.get_entity().entity_id);
        }
    });
    links
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, Ordering::Relaxed},
        mpsc,
    };
    use std::time::Duration;

    use pumpkin_data::{dimension::Dimension, entity::EntityType};
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::{
        entity::{Entity, ai::test_support::server, passive::cow::CowEntity},
        plugin::{
            BoxFuture, EventRwHandler,
            api::events::{EventPriority, entity::entity_target::EntityTargetEvent},
        },
        server::Server,
    };

    struct ReviewTarget {
        replacement: i32,
        candidate: i32,
        calls: Mutex<Vec<(TargetReason, Option<i32>)>>,
        read_completed: AtomicBool,
    }

    impl EventRwHandler<EntityTargetEvent> for ReviewTarget {
        fn handle<'a>(
            &'a self,
            server: &'a Arc<Server>,
            event: &'a mut EntityTargetEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
                let Some(creaking) = world.get_entity_by_id(event.entity_id) else {
                    return;
                };
                let (tx, rx) = mpsc::channel();
                // Bound the read so a callback under the Brain mutex fails this test cleanly.
                let _reader = std::thread::spawn(move || {
                    let from_brain = creaking
                        .get_mob()
                        .and_then(|mob| mob.get_mob_entity().get_target())
                        .map(|target| target.get_entity().entity_id);
                    let _ = tx.send(from_brain);
                });
                let Ok(target) = rx.recv_timeout(Duration::from_secs(2)) else {
                    return;
                };
                self.read_completed.store(true, Relaxed);
                let mut calls = self
                    .calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                match calls.len() {
                    0 | 3 => event.cancelled = true,
                    1 | 2 => event.target_id = Some(self.replacement),
                    4 => event.target_id = Some(self.candidate),
                    5 => event.target_id = Some(i32::MAX),
                    6 => event.target_id = None,
                    _ => {}
                }
                calls.push((event.reason, target));
            })
        }
    }

    fn plugin_target_test_brain(candidate: Arc<dyn EntityBase>, forget: Arc<AtomicBool>) -> Brain {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(start_attacking::create_with_reason(
                    |_| true,
                    move |_| Some(candidate.clone()),
                    |_| TargetReason::ClosestEntity,
                )),
                Box::new(stop_attacking_if_target_invalid::create_with_target_events(
                    move |_, _| forget.load(Relaxed),
                    |_, _| {},
                )),
            ],
        ));
        brain.set(types::CANT_REACH_WALK_TARGET_SINCE, 123);
        brain
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn brain_target_events_see_the_previous_target_and_honor_veto_and_redirection()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let entity = |x, kind| Entity::new(world.clone(), Vector3::new(x, 64.0, 0.0), kind);
        let creaking = CreakingEntity::new(entity(0.0, &EntityType::CREAKING));
        let candidate = CowEntity::new(entity(2.0, &EntityType::COW));
        let replacement = CowEntity::new(entity(3.0, &EntityType::COW));
        world.add_entity_silent(creaking.clone());
        world.add_entity_silent(candidate.clone());
        world.add_entity_silent(replacement.clone());
        let replacement_id = replacement.get_entity().entity_id;
        let candidate_id = candidate.get_entity().entity_id;
        let review = Arc::new(ReviewTarget {
            replacement: replacement_id,
            candidate: candidate_id,
            calls: Mutex::new(Vec::new()),
            read_completed: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<EntityTargetEvent, _>(review.clone(), EventPriority::Normal, None);

        let forget = Arc::new(AtomicBool::new(false));
        let brain = plugin_target_test_brain(candidate, forget.clone());
        let mob = &creaking.mob_entity;
        mob.set_brain(brain);
        let visible_target = || mob.get_target().map(|target| target.get_entity().entity_id);
        let unreachable_since = || {
            mob.with_brain(creaking.as_ref(), |brain| {
                brain.get(types::CANT_REACH_WALK_TARGET_SINCE).copied()
            })
        };

        mob.tick_brain(creaking.as_ref());
        assert_eq!(visible_target(), None);
        assert_eq!(unreachable_since(), Some(123));
        replacement.get_entity().invulnerable.store(true, Relaxed);
        assert!(
            creaking
                .as_valid_target(Some(replacement.clone()))
                .is_none()
        );
        mob.tick_brain(creaking.as_ref());
        assert!(mob.stored_target().is_none());
        assert_eq!(unreachable_since(), Some(123));

        replacement.get_entity().invulnerable.store(false, Relaxed);
        mob.tick_brain(creaking.as_ref());
        assert_eq!(visible_target(), Some(replacement_id));
        assert_eq!(unreachable_since(), None);

        replacement.get_entity().invulnerable.store(true, Relaxed);
        assert_eq!(visible_target(), None);
        forget.store(true, Relaxed);
        mob.with_brain(creaking.as_ref(), |brain| {
            brain.set(types::CANT_REACH_WALK_TARGET_SINCE, 456);
        });
        mob.tick_brain(creaking.as_ref());
        assert_eq!(
            mob.stored_target()
                .map(|target| target.get_entity().entity_id),
            Some(replacement_id)
        );
        assert_eq!(unreachable_since(), Some(456));
        mob.tick_brain(creaking.as_ref());
        assert_eq!(visible_target(), Some(candidate_id));
        assert_eq!(unreachable_since(), Some(456));

        assert!(world.get_entity_by_id(i32::MAX).is_none());
        mob.tick_brain(creaking.as_ref());
        assert_eq!(
            mob.stored_target()
                .map(|target| target.get_entity().entity_id),
            Some(candidate_id)
        );
        assert_eq!(unreachable_since(), Some(456));

        mob.tick_brain(creaking.as_ref());
        assert!(mob.stored_target().is_none());
        assert_eq!(unreachable_since(), Some(456));
        server.shutdown().await;

        assert!(review.read_completed.load(Relaxed));
        let calls = review
            .calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(
            *calls,
            [
                (TargetReason::ClosestEntity, None),
                (TargetReason::ClosestEntity, None),
                (TargetReason::ClosestEntity, None),
                (TargetReason::ForgotTarget, None),
                (TargetReason::ForgotTarget, None),
                (TargetReason::ForgotTarget, Some(candidate_id)),
                (TargetReason::ForgotTarget, Some(candidate_id)),
            ],
        );
        Ok(())
    }
}
