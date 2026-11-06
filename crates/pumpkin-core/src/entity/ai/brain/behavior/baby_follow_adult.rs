use std::sync::Arc;

use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::OneShot;
use crate::entity::mob::Mob;

#[must_use]
pub fn create(min_range: i32, max_range: i32, speed_modifier: f32) -> OneShot {
    create_with(min_range, max_range, move |_| speed_modifier)
}

/// Vanilla `BabyFollowAdult.create(followRange, speedModifier, ...)` with a speed that depends on
/// the mob, as the function overload has it.
#[must_use]
pub fn create_with(
    min_range: i32,
    max_range: i32,
    speed_modifier: impl Fn(&dyn Mob) -> f32 + Send + Sync + 'static,
) -> OneShot {
    OneShot::new(
        "BabyFollowAdult",
        vec![
            (
                types::NEAREST_VISIBLE_ADULT.id(),
                MemoryStatus::ValuePresent,
            ),
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
        ],
        move |tick| {
            let living = &tick.mob.get_mob_entity().living_entity;
            if !living.is_baby() {
                return false;
            }
            let Some(adult) = tick.state.get(types::NEAREST_VISIBLE_ADULT).cloned() else {
                return false;
            };
            let distance = living
                .entity
                .pos
                .load()
                .squared_distance_to_vec(&adult.get_entity().pos.load());
            let max = f64::from(max_range + 1);
            let min = f64::from(min_range);
            if distance >= max * max || distance < min * min {
                return false;
            }
            tick.state.set(
                types::LOOK_TARGET,
                Arc::new(EntityTracker::new(adult.clone(), true)) as Arc<dyn PositionTracker>,
            );
            tick.state.set(
                types::WALK_TARGET,
                WalkTarget::new(
                    Arc::new(EntityTracker::new(adult, false)),
                    speed_modifier(tick.mob),
                    min_range - 1,
                ),
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
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::brain::behavior::BehaviorControl;
    use crate::entity::ai::brain::{BrainState, BrainTick};
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::cow::CowEntity;
    use crate::entity::{Entity, EntityBase};

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const SPEED_IN_WATER: f32 = 0.6;
    const SPEED_ON_LAND: f32 = 0.15;

    /// The speed and the walk target distance a baby cow 8 blocks from its adult follows it with.
    fn follows(
        mut behavior: OneShot,
        in_water: bool,
    ) -> Result<Option<(f32, i32)>, Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let baby = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COW,
        ));
        baby.get_entity().age.store(-24000, Ordering::Relaxed);
        baby.get_entity()
            .touching_water
            .store(in_water, Ordering::Relaxed);
        let adult: Arc<dyn EntityBase> = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(8.5, 64.0, 0.5),
            &EntityType::COW,
        ));
        let mut state = BrainState::default();
        for id in behavior.required_memories() {
            state.register_memory(*id);
        }
        state.set(types::NEAREST_VISIBLE_ADULT, adult);
        behavior.try_start(&mut BrainTick {
            state: &mut state,
            world: &world,
            mob: &*baby,
            time: 0,
        });
        Ok(state
            .get(types::WALK_TARGET)
            .map(|walk| (walk.speed_modifier(), walk.close_enough_dist())))
    }

    /// The speed is asked of the mob when the target is set (water or land); the plain form keeps
    /// the one speed it was given. Both stop a minimum range short of the adult.
    #[tokio::test]
    async fn the_follow_speed_can_depend_on_the_mob() -> TestResult {
        let by_mob = |in_water: bool| {
            follows(
                create_with(5, 16, |mob| {
                    if mob.get_entity().is_in_water() {
                        SPEED_IN_WATER
                    } else {
                        SPEED_ON_LAND
                    }
                }),
                in_water,
            )
        };

        assert_eq!(by_mob(true)?, Some((SPEED_IN_WATER, 4)));
        assert_eq!(by_mob(false)?, Some((SPEED_ON_LAND, 4)));
        assert_eq!(follows(create(5, 16, 2.5), true)?, Some((2.5, 4)));
        Ok(())
    }
}
