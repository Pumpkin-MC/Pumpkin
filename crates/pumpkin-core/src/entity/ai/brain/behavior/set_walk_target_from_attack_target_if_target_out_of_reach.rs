use super::super::memory::{MemoryStatus, types, walk_target::WalkTarget};
use super::{OneShot, utils};
use crate::entity::mob::Mob;

/// The walk target of vanilla `SetWalkTargetFromAttackTargetIfTargetOutOfReach`: right on the
/// target.
const CLOSE_ENOUGH_DISTANCE: i32 = 0;

/// Vanilla `SetWalkTargetFromAttackTargetIfTargetOutOfReach`: walks to the attack target at
/// `speed_modifier` while it is out of reach or out of sight, and stands still once it is in
/// reach.
#[must_use]
pub fn create(speed_modifier: fn(&dyn Mob) -> f32) -> OneShot {
    OneShot::new(
        "SetWalkTargetFromAttackTargetIfTargetOutOfReach",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::Registered,
            ),
        ],
        move |tick| {
            let Some(target) = tick.state.get(types::ATTACK_TARGET).cloned() else {
                return false;
            };
            if utils::entity_is_visible(&tick.visibility(), target.as_ref())
                && utils::is_within_attack_range(tick.mob, target.as_ref())
            {
                tick.state.erase(types::WALK_TARGET.id());
            } else {
                utils::look_at_entity(tick.state, target.clone());
                tick.state.set(
                    types::WALK_TARGET,
                    WalkTarget::from_entity(
                        target,
                        speed_modifier(tick.mob),
                        CLOSE_ENOUGH_DISTANCE,
                    ),
                );
            }
            true
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
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::brain::memory::NearestVisibleLivingEntities;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::entity::{Entity, EntityBase};
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const IN_WATER: f32 = 0.6;
    const ON_LAND: f32 = 0.15;

    fn chasing_speed(mob: &dyn Mob) -> f32 {
        if mob.get_entity().is_in_water() {
            IN_WATER
        } else {
            ON_LAND
        }
    }

    fn cow(world: &Arc<World>, x: f64) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::COW,
        ))
    }

    fn brain_for(target: &Arc<CowEntity>, visible: bool) -> Brain {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create(chasing_speed))],
        ));
        let target: Arc<dyn EntityBase> = target.clone();
        brain.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(if visible {
                vec![target.clone()]
            } else {
                Vec::new()
            }),
        );
        brain.set(types::ATTACK_TARGET, target);
        brain
    }

    fn walk_target_of(brain: &Brain) -> Option<(i32, f32, i32)> {
        let walk = brain.get(types::WALK_TARGET)?;
        let tracked = walk.target().tracked_entity()?.get_entity().entity_id;
        Some((tracked, walk.speed_modifier(), walk.close_enough_dist()))
    }

    /// Out of reach it looks at the target and walks to it at the chasing speed of the water it
    /// is in; once the target is in reach the walk target is dropped.
    #[tokio::test]
    async fn walks_to_a_target_out_of_reach_and_stops_once_it_is_in_reach() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 5.0);
        let mut brain = brain_for(&prey, true);
        let id = prey.get_entity().entity_id;

        brain.tick(&world, &*body, 1);
        let on_land = walk_target_of(&brain);
        let looks_at_it = brain
            .get(types::LOOK_TARGET)
            .and_then(|look| look.tracked_entity())
            .map(|entity| entity.get_entity().entity_id);

        body.get_entity()
            .touching_water
            .store(true, std::sync::atomic::Ordering::Relaxed);
        brain.erase(types::WALK_TARGET.id());
        brain.tick(&world, &*body, 2);
        let in_water = walk_target_of(&brain);

        prey.get_entity().set_pos(Vector3::new(1.5, 64.0, 0.0));
        brain.tick(&world, &*body, 3);
        let in_reach = brain.has_memory_value(types::WALK_TARGET.id());
        world.level.shutdown().await;

        assert_eq!(on_land, Some((id, ON_LAND, 0)));
        assert_eq!(looks_at_it, Some(id));
        assert_eq!(in_water, Some((id, IN_WATER, 0)));
        assert!(!in_reach);
        Ok(())
    }

    /// A target in reach that the mob cannot see (it is not among the visible entities) is still
    /// walked to.
    #[tokio::test]
    async fn keeps_walking_to_a_target_in_reach_that_it_cannot_see() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = cow(&world, 0.0);
        let prey = cow(&world, 1.5);
        let mut brain = brain_for(&prey, false);

        brain.tick(&world, &*body, 1);
        let walks = walk_target_of(&brain);
        world.level.shutdown().await;

        assert_eq!(
            walks.map(|(id, _, _)| id),
            Some(prey.get_entity().entity_id)
        );
        Ok(())
    }
}
