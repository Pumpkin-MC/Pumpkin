use std::sync::Arc;

use pumpkin_util::math::vector3::Vector3;

use super::super::OneShot;
use crate::entity::EntityBase;
use crate::entity::ai::brain::memory::{
    MemoryModuleType, MemoryStatus, types, walk_target::WalkTarget,
};
use crate::entity::ai::util::land_random_pos;

const HORIZONTAL_DISTANCE: i32 = 16;
const VERTICAL_DISTANCE: i32 = 7;
const ATTEMPTS: i32 = 10;

fn walk_is_away(
    current_position: Vector3<f64>,
    body_position: Vector3<f64>,
    avoid_position: Vector3<f64>,
) -> bool {
    (current_position - body_position).dot(&(avoid_position - body_position)) >= 0.0
}

#[must_use]
pub fn entity(
    memory: MemoryModuleType<Arc<dyn EntityBase>>,
    speed_modifier: f32,
    desired_distance: i32,
    interrupt_current_walk: bool,
) -> OneShot {
    OneShot::new(
        "SetWalkTargetAwayFrom",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::Registered),
            (memory.id(), MemoryStatus::ValuePresent),
        ],
        move |tick| {
            let walk_target_present = tick.state.get(types::WALK_TARGET).is_some();
            if walk_target_present && !interrupt_current_walk {
                return false;
            }
            let body_position = tick.mob.get_entity().pos.load();
            let Some(avoid_position) = tick
                .state
                .get(memory)
                .map(|entity| entity.get_entity().pos.load())
            else {
                return false;
            };
            if body_position.squared_distance_to_vec(&avoid_position)
                >= f64::from(desired_distance) * f64::from(desired_distance)
            {
                return false;
            }
            let current_position = tick.state.get(types::WALK_TARGET).and_then(|target| {
                (target.speed_modifier() == speed_modifier)
                    .then(|| target.target().current_position())
            });
            if current_position
                .is_some_and(|current| !walk_is_away(current, body_position, avoid_position))
            {
                return false;
            }
            for _ in 0..ATTEMPTS {
                if let Some(flee_to) = land_random_pos::get_pos_away(
                    tick.mob,
                    HORIZONTAL_DISTANCE,
                    VERTICAL_DISTANCE,
                    avoid_position,
                ) {
                    tick.state.set(
                        types::WALK_TARGET,
                        WalkTarget::from_vec(flee_to, speed_modifier, 0),
                    );
                    break;
                }
            }
            true
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_is_away_accepts_a_perpendicular_or_forward_walk() {
        let body = Vector3::new(0.0, 0.0, 0.0);
        let avoid = Vector3::new(2.0, 0.0, 0.0);
        assert!(walk_is_away(Vector3::new(1.0, 0.0, 0.0), body, avoid));
        assert!(walk_is_away(Vector3::new(0.0, 0.0, 1.0), body, avoid));
        assert!(!walk_is_away(Vector3::new(-1.0, 0.0, 0.0), body, avoid));
    }
}
