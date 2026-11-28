use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::EntityBase;
use crate::plugin::api::events::entity::entity_target::TargetReason;

const TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE: i64 = 200;

/// Drops the attack target once it can no longer be attacked, died, left the world or
/// stayed out of reach for too long.
#[must_use]
pub fn create() -> OneShot {
    create_with_callback(|_, _| {})
}

#[must_use]
pub fn create_with_callback(
    on_target_erased: impl Fn(&BrainTick<'_>, &Arc<dyn EntityBase>) + Send + Sync + 'static,
) -> OneShot {
    create_with(|_, _| false, on_target_erased, false)
}

/// Proposes forgetting the target through the plugin event.
/// The mob must opt into Brain-owned targets. This bypasses `Mob::apply_mob_target`.
#[must_use]
pub fn create_with_target_events(
    is_invalid_target: impl FnMut(&BrainTick<'_>, &Arc<dyn EntityBase>) -> bool + Send + Sync + 'static,
    on_target_erased: impl Fn(&BrainTick<'_>, &Arc<dyn EntityBase>) + Send + Sync + 'static,
) -> OneShot {
    create_with(is_invalid_target, on_target_erased, true)
}

fn create_with(
    mut is_invalid_target: impl FnMut(&BrainTick<'_>, &Arc<dyn EntityBase>) -> bool
    + Send
    + Sync
    + 'static,
    on_target_erased: impl Fn(&BrainTick<'_>, &Arc<dyn EntityBase>) + Send + Sync + 'static,
    fire_target_event: bool,
) -> OneShot {
    let on_target_erased = Arc::new(on_target_erased);
    OneShot::new(
        "StopAttackingIfTargetInvalid",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (
                types::CANT_REACH_WALK_TARGET_SINCE.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            let Some(target) = tick.state.get(types::ATTACK_TARGET).cloned() else {
                return false;
            };
            let tired = tick
                .state
                .get(types::CANT_REACH_WALK_TARGET_SINCE)
                .is_some_and(|since| tick.time - since > TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE);
            let still_valid = tick.mob.can_attack(target.as_ref())
                && !tired
                && target
                    .get_living_entity()
                    .is_some_and(crate::entity::living::LivingEntity::is_alive)
                && Arc::ptr_eq(&target.get_entity().world.load_full(), tick.world)
                && !is_invalid_target(tick, &target);
            if !still_valid {
                if fire_target_event {
                    let on_target_erased = on_target_erased.clone();
                    tick.defer_target_change(
                        None,
                        TargetReason::ForgotTarget,
                        move |tick, approved| {
                            if !tick
                                .state
                                .get(types::ATTACK_TARGET)
                                .is_some_and(|current| Arc::ptr_eq(current, &target))
                            {
                                return;
                            }
                            if approved.is_none() {
                                on_target_erased(tick, &target);
                            }
                            tick.state.set_optional(types::ATTACK_TARGET, approved);
                        },
                    );
                } else {
                    on_target_erased(tick, &target);
                    tick.state.erase(types::ATTACK_TARGET.id());
                }
            }
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::Ordering;

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::test_support::{server, world};
    use crate::entity::passive::cow::CowEntity;
    use crate::plugin::{
        BoxFuture, EventRwHandler,
        api::events::{EventPriority, entity::entity_target::EntityTargetEvent},
    };
    use crate::server::Server;
    use crate::world::World;
    use pumpkin_data::dimension::Dimension;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    type Erased = Arc<Mutex<Vec<i32>>>;

    fn cow(world: &Arc<World>, x: f64) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::COW,
        ))
    }

    fn attacking(target: Arc<dyn EntityBase>, fire_target_event: bool) -> (Brain, Erased) {
        let erased = Erased::default();
        let record = erased.clone();
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create_with(
                |_, _| false,
                move |tick, target| {
                    let in_memory = tick
                        .state
                        .get(types::ATTACK_TARGET)
                        .map(|held| held.get_entity().entity_id);
                    assert_eq!(in_memory, Some(target.get_entity().entity_id));
                    record
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(target.get_entity().entity_id);
                },
                fire_target_event,
            ))],
        ));
        brain.set(types::ATTACK_TARGET, target);
        (brain, erased)
    }

    fn erased_ids(erased: &Erased) -> Vec<i32> {
        erased
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[tokio::test]
    async fn keeps_a_valid_target_and_drops_a_dead_or_unattackable_one() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 2.0);
        let (mut brain, erased) = attacking(prey.clone(), false);

        brain.tick(&world, &*body, 1);
        let kept = brain.has_memory_value(types::ATTACK_TARGET.id());
        let reported_while_valid = erased_ids(&erased);

        prey.get_entity()
            .invulnerable
            .store(true, Ordering::Relaxed);
        brain.tick(&world, &*body, 2);
        let dropped_when_unattackable = !brain.has_memory_value(types::ATTACK_TARGET.id());

        prey.get_entity()
            .invulnerable
            .store(false, Ordering::Relaxed);
        brain.set(types::ATTACK_TARGET, prey.clone());
        prey.mob_entity.living_entity.health.store(0.0);
        brain.tick(&world, &*body, 3);
        let dropped_when_dead = !brain.has_memory_value(types::ATTACK_TARGET.id());
        world.level.shutdown().await;

        assert!(kept);
        assert!(reported_while_valid.is_empty());
        assert!(dropped_when_unattackable);
        assert!(dropped_when_dead);
        let id = prey.get_entity().entity_id;
        assert_eq!(erased_ids(&erased), vec![id, id]);
        Ok(())
    }

    struct ReplaceTarget {
        body: Arc<CowEntity>,
        replacement: Arc<dyn EntityBase>,
    }

    impl EventRwHandler<EntityTargetEvent> for ReplaceTarget {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityTargetEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                assert_eq!(event.reason, TargetReason::ForgotTarget);
                assert_eq!(event.target_id, None);
                self.body
                    .mob_entity
                    .with_brain(self.body.as_ref(), |brain| {
                        brain.set(types::ATTACK_TARGET, self.replacement.clone());
                    });
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn forget_does_not_erase_target_installed_by_the_listener() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 2.0);
        let replacement = cow(&world, 3.0);
        let (brain, erased) = attacking(prey.clone(), true);
        body.mob_entity.set_brain(brain);
        prey.get_entity()
            .invulnerable
            .store(true, Ordering::Relaxed);
        server.plugin_manager.register_blocking(
            Arc::new(ReplaceTarget {
                body: body.clone(),
                replacement: replacement.clone(),
            }),
            EventPriority::Normal,
            None,
        );

        body.mob_entity.tick_brain(body.as_ref());

        assert!(erased_ids(&erased).is_empty());
        assert_eq!(
            body.mob_entity.with_brain(body.as_ref(), |brain| {
                brain
                    .get(types::ATTACK_TARGET)
                    .map(|target| target.get_entity().entity_id)
            }),
            Some(replacement.get_entity().entity_id),
        );
        server.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn approved_forget_runs_callback_before_erasing_and_keeps_reach_timer() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 2.0);
        let (mut brain, erased) = attacking(prey.clone(), true);
        brain.set(types::CANT_REACH_WALK_TARGET_SINCE, 1);
        prey.get_entity()
            .invulnerable
            .store(true, Ordering::Relaxed);

        brain.tick(&world, body.as_ref(), 2);

        assert_eq!(erased_ids(&erased), vec![prey.get_entity().entity_id]);
        assert!(brain.get(types::ATTACK_TARGET).is_none());
        assert_eq!(brain.get(types::CANT_REACH_WALK_TARGET_SINCE), Some(&1));
        world.level.shutdown().await;
        Ok(())
    }
}
