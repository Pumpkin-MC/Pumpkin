use std::sync::Arc;

use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::EntityBase;

/// How long the target may stay out of reach before the mob gives up (vanilla
/// `TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE`).
const TIMEOUT_TO_GET_WITHIN_ATTACK_RANGE: i64 = 200;

/// Vanilla `StopAttackingIfTargetInvalid.create()`: drops the attack target once it can no longer
/// be attacked, died, left the world or stayed out of reach for too long.
#[must_use]
pub fn create() -> OneShot {
    create_with_callback(|_, _| {})
}

/// Vanilla `StopAttackingIfTargetInvalid.create(onTargetErased)`.
///
/// Like [`create`], and tells `on_target_erased` about the target first (the target is still in
/// the memory then). Vanilla's other overloads (an extra stop condition, and giving up that can
/// be turned off) are left out because nothing here needs them.
#[must_use]
pub fn create_with_callback(
    on_target_erased: impl Fn(&BrainTick<'_>, &Arc<dyn EntityBase>) + Send + Sync + 'static,
) -> OneShot {
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
                && Arc::ptr_eq(&target.get_entity().world.load_full(), tick.world);
            if !still_valid {
                on_target_erased(tick, &target);
                tick.state.erase(types::ATTACK_TARGET.id());
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
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;
    type Erased = Arc<Mutex<Vec<i32>>>;

    fn cow(world: &Arc<World>, x: f64) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::COW,
        ))
    }

    /// A brain that records the id of every target it is told about, holding `target`.
    fn attacking(target: Arc<dyn EntityBase>) -> (Brain, Erased) {
        let erased = Erased::default();
        let record = erased.clone();
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create_with_callback(move |tick, target| {
                // The target is still in the memory while the callback runs.
                let in_memory = tick
                    .state
                    .get(types::ATTACK_TARGET)
                    .map(|held| held.get_entity().entity_id);
                assert_eq!(in_memory, Some(target.get_entity().entity_id));
                record
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(target.get_entity().entity_id);
            }))],
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

    /// A target the mob may attack and can reach stays; a dead one and one that may not be
    /// attacked go, each reported once to the callback before the memory is erased.
    #[tokio::test]
    async fn keeps_a_valid_target_and_drops_a_dead_or_unattackable_one() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 2.0);
        let (mut brain, erased) = attacking(prey.clone());

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
}
