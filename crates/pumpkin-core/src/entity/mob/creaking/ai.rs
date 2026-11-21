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
                Box::new(start_attacking::create(
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
                Box::new(stop_attacking_if_target_invalid::create_with_filter(
                    is_target_unreachable,
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
    mob.tick_brain_with_target_events(creaking, TargetReason::ClosestPlayer);
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

fn entity_id(entity: &dyn EntityBase) -> i32 {
    entity.get_entity().entity_id
}

pub(super) fn tick_links(creaking: &CreakingEntity) -> Vec<i32> {
    let mut links = Vec::new();
    let stored = creaking.mob_entity.stored_target();
    links.extend(stored.as_deref().map(entity_id));
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
    use std::{sync::mpsc, time::Duration};

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::{Entity, ai::test_support::world, passive::cow::CowEntity};

    #[tokio::test]
    async fn target_setter_completes_while_the_brain_is_locked()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let creaking = CreakingEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::CREAKING,
        ));
        let previous = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(3.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        creaking.mob_entity.set_target(Some(previous.clone()));
        creaking.mob_entity.with_brain(creaking.as_ref(), |brain| {
            brain.set(types::ATTACK_TARGET, previous);
            brain.set(types::CANT_REACH_WALK_TARGET_SINCE, 123);
        });
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(2.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let (sender, result) = mpsc::channel();
        let completed = std::thread::scope(|scope| {
            creaking.mob_entity.with_brain(creaking.as_ref(), |_| {
                scope.spawn(|| {
                    let updated =
                        creaking.set_mob_target(Some(cow.clone()), TargetReason::ClosestPlayer);
                    sender.send(updated)
                });
                result.recv_timeout(Duration::from_secs(2))
            })
        });
        assert!(completed?);
        creaking.set_can_move(false);
        tick(&creaking);
        creaking.mob_entity.with_brain(creaking.as_ref(), |brain| {
            assert_eq!(
                brain
                    .get(types::ATTACK_TARGET)
                    .map(|target| entity_id(target.as_ref())),
                Some(cow.get_entity().entity_id),
            );
            assert!(brain.get(types::CANT_REACH_WALK_TARGET_SINCE).is_none());
            brain.set(types::CANT_REACH_WALK_TARGET_SINCE, 456);
        });
        tick(&creaking);
        creaking.mob_entity.with_brain(creaking.as_ref(), |brain| {
            assert_eq!(brain.get(types::CANT_REACH_WALK_TARGET_SINCE), Some(&456));
        });
        world.level.shutdown().await;
        Ok(())
    }
}
