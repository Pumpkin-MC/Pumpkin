use std::sync::Arc;

use pumpkin_data::{entity::EntityType, environment_attribute::Activity};

use super::{BreezeEntity, movement, shoot};
use crate::entity::EntityBase;
use crate::entity::ai::brain::{
    Brain, BrainProvider, BrainState, BrainTick,
    activity::ActivityData,
    behavior::{
        DoNothing, LookAtTargetSink, OneShot, Swim, Timed, random_stroll, run_one, start_attacking,
        stop_attacking_if_target_invalid,
    },
    memory::{MemoryStatus, PackedMemories, types},
    sensing::{SensorType, is_entity_attackable, remember_positives},
};
use crate::entity::mob::Mob;
use crate::plugin::api::events::entity::entity_target::TargetReason;

const TICKS_TO_REMEMBER_SEEN_TARGET: i32 = 100;

pub(super) fn make_brain(breeze: &BreezeEntity, packed: &PackedMemories) -> Brain {
    let mut brain = BrainProvider {
        memory_types: &[],
        sensor_types: &[
            SensorType::HurtBy,
            SensorType::NearestPlayers,
            SensorType::BreezeAttackEntity,
        ],
        activities,
    }
    .make_brain(breeze, packed);
    brain.set_default_activity(Activity::Fight);
    brain.use_default_activity();
    brain
}

pub(super) fn update_activity(brain: &mut BrainState) {
    brain.set_active_activity_to_first_valid(&[Activity::Fight, Activity::Idle]);
}

fn activities(_mob: &dyn Mob) -> Vec<ActivityData> {
    vec![
        ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![
                Box::new(Timed::new(Swim::new(0.8))),
                Box::new(Timed::new(LookAtTargetSink::new(45, 90))),
            ],
        ),
        ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![
                Box::new(start_attacking::create_with_reason(
                    |_| true,
                    nearest_attackable,
                    |target| {
                        if target.get_entity().entity_type == &EntityType::PLAYER {
                            TargetReason::ClosestPlayer
                        } else {
                            TargetReason::ClosestEntity
                        }
                    },
                )),
                Box::new(start_attacking::create_with_reason(
                    |_| true,
                    hurt_by,
                    |_| TargetReason::TargetAttackedEntity,
                )),
                movement::slide_to_target_sink(),
                Box::new(run_one(vec![
                    (Box::new(DoNothing::new(20, 100)), 1),
                    (Box::new(random_stroll::stroll(0.6)), 2),
                ])),
            ],
        ),
        ActivityData::with_priority_start_and_conditions(
            Activity::Fight,
            0,
            vec![
                Box::new(stop_attacking()),
                shoot::shoot(),
                movement::long_jump(),
                shoot::shoot_when_stuck(),
                movement::slide(),
            ],
            vec![
                (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
                (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            ],
        ),
    ]
}

fn nearest_attackable(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    tick.state.get(types::NEAREST_ATTACKABLE).cloned()
}

fn hurt_by(tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
    let target = tick.state.get(types::HURT_BY)?.attacker.clone()?;
    target.get_living_entity()?;
    Some(target)
}

fn stop_attacking() -> OneShot {
    let mut recently_attackable = remember_positives(TICKS_TO_REMEMBER_SEEN_TARGET, |&value| value);
    stop_attacking_if_target_invalid::create_with_target_events(move |tick, target| {
        !recently_attackable(&is_entity_attackable(&tick.visibility(), target.as_ref()))
    })
}

pub(super) fn forget_target(tick: &mut BrainTick<'_>) {
    tick.defer(|mob| {
        mob.set_mob_target(None, TargetReason::ForgotTarget);
    });
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    use futures::future::BoxFuture;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::{Entity, ai::test_support, r#type::from_type};
    use crate::plugin::api::events::entity::entity_target::EntityTargetEvent;
    use crate::plugin::{EventHandler, EventPriority};
    use crate::server::Server;

    struct TargetListener {
        breeze: Arc<BreezeEntity>,
        replacement: i32,
        acquisitions: AtomicUsize,
        clears: AtomicUsize,
    }

    impl EventHandler<EntityTargetEvent> for TargetListener {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityTargetEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.entity_id != self.breeze.get_entity().entity_id {
                    return;
                }
                let has_target = self
                    .breeze
                    .mob_entity
                    .brain_has_memory_value(types::ATTACK_TARGET.id());
                assert!(matches!(
                    event.reason,
                    TargetReason::ClosestEntity | TargetReason::ForgotTarget
                ));
                match event.reason {
                    TargetReason::ClosestEntity => {
                        assert!(!has_target);
                        if self.acquisitions.fetch_add(1, Relaxed) == 0 {
                            event.cancelled = true;
                        } else {
                            event.target_id = Some(self.replacement);
                        }
                    }
                    TargetReason::ForgotTarget => {
                        assert!(has_target);
                        event.cancelled = self.clears.fetch_add(1, Relaxed) == 0;
                    }
                    _ => (),
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn automatic_target_changes_allow_plugin_veto_and_redirection()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = test_support::server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::BREEZE,
        ));
        let iron_golem = |x| {
            from_type(
                &EntityType::IRON_GOLEM,
                Vector3::new(x, 64.0, 0.5),
                &world,
                uuid::Uuid::new_v4(),
            )
        };
        let candidate = iron_golem(6.5);
        let replacement = iron_golem(9.5);
        assert!(world.spawn_entity(replacement.clone()));
        let listener = Arc::new(TargetListener {
            breeze: breeze.clone(),
            replacement: replacement.get_entity().entity_id,
            acquisitions: AtomicUsize::new(0),
            clears: AtomicUsize::new(0),
        });
        server.plugin_manager.register::<EntityTargetEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            true,
        );

        // Install the production behaviors with a fixed sensor result.
        let body = &breeze.mob_entity;
        let mut brain = Brain::default();
        for activity in activities(breeze.as_ref()) {
            brain.add_activity(activity);
        }
        brain.register_memory(types::NEAREST_ATTACKABLE.id());
        brain.set(types::NEAREST_ATTACKABLE, candidate);
        body.set_brain(brain);

        breeze.custom_server_ai_step(breeze.as_ref());
        assert_eq!(listener.acquisitions.load(Relaxed), 1);
        assert!(!body.brain_has_memory_value(types::ATTACK_TARGET.id()));

        breeze.custom_server_ai_step(breeze.as_ref());
        assert_eq!(listener.acquisitions.load(Relaxed), 2);
        assert!(
            body.get_target()
                .is_some_and(|target| Arc::ptr_eq(&target, &replacement))
        );

        replacement.get_entity().invulnerable.store(true, Relaxed);
        assert!(body.get_target().is_none());
        breeze.custom_server_ai_step(breeze.as_ref());
        assert_eq!(listener.clears.load(Relaxed), 1);
        assert!(body.brain_has_memory_value(types::ATTACK_TARGET.id()));

        breeze.custom_server_ai_step(breeze.as_ref());
        assert_eq!(listener.clears.load(Relaxed), 2);
        assert!(!body.brain_has_memory_value(types::ATTACK_TARGET.id()));

        server.stop();
        server.shutdown().await;
        Ok(())
    }
}
