use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, MemoryStatus};
use super::OneShot;

#[must_use]
pub fn create(
    predicate: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    memory: MemoryModuleId,
) -> OneShot {
    OneShot::new(
        "EraseMemoryIf",
        vec![(memory, MemoryStatus::ValuePresent)],
        move |tick| {
            if predicate(tick) {
                tick.state.erase(memory);
                true
            } else {
                false
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::brain::behavior::utils;
    use crate::entity::ai::brain::memory::types;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn erases_the_memory_only_while_the_predicate_holds() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let partner: Arc<dyn crate::entity::EntityBase> = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(1.0, 64.0, 0.0),
            &EntityType::COW,
        ));
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create(
                |tick| utils::is_breeding(tick.state),
                types::PLAY_DEAD_TICKS.id(),
            ))],
        ));
        brain.register_memory(types::BREED_TARGET.id());
        brain.set(types::PLAY_DEAD_TICKS, 150);

        brain.tick(&world, &*cow, 1);
        let kept_without_a_partner = brain.get(types::PLAY_DEAD_TICKS).copied();
        brain.set(types::BREED_TARGET, partner);
        brain.tick(&world, &*cow, 2);
        let erased_while_breeding = brain.has_memory_value(types::PLAY_DEAD_TICKS.id());
        world.level.shutdown().await;

        assert_eq!(kept_without_a_partner, Some(150));
        assert!(!erased_while_breeding);
        Ok(())
    }
}
