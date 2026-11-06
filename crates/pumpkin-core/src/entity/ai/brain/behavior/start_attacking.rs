use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::EntityBase;

/// Vanilla `StartAttacking`: with no attack target, takes the one `target_finder` offers when the
/// mob may attack it, and forgets that it could not reach its walk target.
#[must_use]
pub fn create(
    target_finder: impl Fn(&BrainTick<'_>) -> Option<Arc<dyn EntityBase>> + Send + Sync + 'static,
) -> OneShot {
    OneShot::new(
        "StartAttacking",
        vec![
            (types::ATTACK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::CANT_REACH_WALK_TARGET_SINCE.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            let Some(target) = target_finder(tick) else {
                return false;
            };
            if !tick.mob.can_attack(target.as_ref()) {
                return false;
            }
            tick.state.set(types::ATTACK_TARGET, target);
            tick.state.erase(types::CANT_REACH_WALK_TARGET_SINCE.id());
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn cow(world: &Arc<World>, x: f64) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::COW,
        ))
    }

    fn brain_with(
        finder: impl Fn(&BrainTick<'_>) -> Option<Arc<dyn EntityBase>> + Send + Sync + 'static,
    ) -> Brain {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create(finder))],
        ));
        brain
    }

    /// Nothing to attack, something it may not attack, and something it may attack: only the
    /// last one becomes the attack target, and it also clears the unreachable walk target mark.
    #[tokio::test]
    async fn takes_the_found_target_only_when_it_may_be_attacked() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 2.0);
        let offered: Arc<std::sync::Mutex<Option<Arc<dyn EntityBase>>>> = Arc::default();
        let finder_offer = offered.clone();
        let mut brain = brain_with(move |_| {
            finder_offer
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        });
        brain.set(types::CANT_REACH_WALK_TARGET_SINCE, 5);

        brain.tick(&world, &*body, 1);
        let nothing_offered = brain.has_memory_value(types::ATTACK_TARGET.id());

        prey.get_entity()
            .invulnerable
            .store(true, Ordering::Relaxed);
        *offered
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(prey.clone());
        brain.tick(&world, &*body, 2);
        let refused = brain.has_memory_value(types::ATTACK_TARGET.id());
        let kept_the_mark = brain.has_memory_value(types::CANT_REACH_WALK_TARGET_SINCE.id());

        prey.get_entity()
            .invulnerable
            .store(false, Ordering::Relaxed);
        brain.tick(&world, &*body, 3);
        let target = brain
            .get(types::ATTACK_TARGET)
            .map(|target| target.get_entity().entity_id);
        let cleared_the_mark = !brain.has_memory_value(types::CANT_REACH_WALK_TARGET_SINCE.id());
        world.level.shutdown().await;

        assert!(!nothing_offered);
        assert!(!refused);
        assert!(kept_the_mark);
        assert_eq!(target, Some(prey.get_entity().entity_id));
        assert!(cleared_the_mark);
        Ok(())
    }

    /// With an attack target the finder is not even asked.
    #[tokio::test]
    async fn does_nothing_while_there_is_an_attack_target() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let current: Arc<dyn EntityBase> = cow(&world, 2.0);
        let asked = Arc::new(AtomicUsize::new(0));
        let counter = asked.clone();
        let mut brain = brain_with(move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
            None
        });
        brain.register_memory(types::ATTACK_TARGET.id());
        brain.set(types::ATTACK_TARGET, current);

        brain.tick(&world, &*body, 1);
        world.level.shutdown().await;

        assert_eq!(asked.load(Ordering::Relaxed), 0);
        Ok(())
    }
}
