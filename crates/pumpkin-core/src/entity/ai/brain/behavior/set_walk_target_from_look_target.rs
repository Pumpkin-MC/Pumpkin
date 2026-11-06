use super::super::BrainTick;
use super::super::memory::{MemoryStatus, types, walk_target::WalkTarget};
use super::OneShot;
use crate::entity::mob::Mob;

#[must_use]
pub fn create(speed_modifier: f32, close_enough_distance: i32) -> OneShot {
    create_with(|_| true, move |_| speed_modifier, close_enough_distance)
}

/// Vanilla `SetWalkTargetFromLookTarget.create(canSetWalkTargetPredicate, speedModifier, ...)`:
/// the walk target is only set when `can_set_walk_target` agrees, at a speed that depends on the
/// mob.
#[must_use]
pub fn create_with(
    can_set_walk_target: impl Fn(&BrainTick<'_>) -> bool + Send + Sync + 'static,
    speed_modifier: impl Fn(&dyn Mob) -> f32 + Send + Sync + 'static,
    close_enough_distance: i32,
) -> OneShot {
    OneShot::new(
        "SetWalkTargetFromLookTarget",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::LOOK_TARGET.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            if !can_set_walk_target(tick) {
                return false;
            }
            let Some(target) = tick.state.get(types::LOOK_TARGET).cloned() else {
                return false;
            };
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(target, speed_modifier(tick.mob), close_enough_distance),
            );
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::brain::behavior::BehaviorControl;
    use crate::entity::ai::brain::memory::PositionTracker;
    use crate::entity::ai::brain::memory::position_tracker::BlockPosTracker;
    use crate::entity::ai::brain::{BrainState, BrainTick};
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::entity::{Entity, EntityBase};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// The speed and distance of the walk target the behavior sets for a cow looking at a block,
    /// if it sets one.
    fn walks(
        mut behavior: OneShot,
        in_water: bool,
    ) -> Result<Option<(f32, i32)>, Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cow = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COW,
        ));
        cow.get_entity()
            .touching_water
            .store(in_water, Ordering::Relaxed);
        let mut state = BrainState::default();
        for id in behavior.required_memories() {
            state.register_memory(*id);
        }
        state.set(
            types::LOOK_TARGET,
            Arc::new(BlockPosTracker::new(BlockPos::new(4, 64, 0))) as Arc<dyn PositionTracker>,
        );
        behavior.try_start(&mut BrainTick {
            state: &mut state,
            world: &world,
            mob: &*cow,
            time: 0,
        });
        Ok(state
            .get(types::WALK_TARGET)
            .map(|walk| (walk.speed_modifier(), walk.close_enough_dist())))
    }

    /// The predicate decides whether the walk target is set and the speed is the mob's; the plain
    /// form always sets it with the speed it was given.
    #[tokio::test]
    async fn the_walk_target_can_depend_on_a_predicate_and_the_mob() -> TestResult {
        let in_water_only = |in_water| {
            walks(
                create_with(
                    |tick| tick.mob.get_entity().is_in_water(),
                    |mob| {
                        if mob.get_entity().is_in_water() {
                            0.5
                        } else {
                            0.15
                        }
                    },
                    3,
                ),
                in_water,
            )
        };

        assert_eq!(in_water_only(true)?, Some((0.5, 3)));
        assert_eq!(in_water_only(false)?, None);
        assert_eq!(walks(create(0.7, 2), false)?, Some((0.7, 2)));
        Ok(())
    }
}
