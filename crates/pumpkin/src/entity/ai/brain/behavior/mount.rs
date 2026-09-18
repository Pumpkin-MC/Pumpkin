use std::sync::Arc;

use crate::entity::EntityBase;

use super::super::BrainTick;
use super::super::memory::position_tracker::{EntityTracker, PositionTracker};
use super::super::memory::walk_target::WalkTarget;
use super::super::memory::{MemoryStatus, types};
use super::one_shot::OneShot;

const CLOSE_ENOUGH_TO_START_RIDING_DIST: f64 = 1.0;

#[must_use]
pub fn mount(speed_modifier: f32) -> OneShot {
    OneShot::with_required(
        "Mount",
        vec![
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::RIDE_TARGET.id(), MemoryStatus::ValuePresent),
        ],
        vec![types::LOOK_TARGET.id()],
        move |tick| {
            if tick.mob.is_passenger() {
                return false;
            }
            let Some(vehicle) = tick.brain.get(types::RIDE_TARGET).cloned() else {
                return false;
            };
            let body_pos = tick.mob.get_entity().pos.load();
            let close_enough = vehicle
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&body_pos)
                < CLOSE_ENOUGH_TO_START_RIDING_DIST * CLOSE_ENOUGH_TO_START_RIDING_DIST;

            if close_enough {
                start_riding(tick, &vehicle);
            } else {
                tick.brain.set(
                    types::LOOK_TARGET,
                    Arc::new(EntityTracker::new(Arc::clone(&vehicle), true))
                        as Arc<dyn PositionTracker>,
                );
                tick.brain.set(
                    types::WALK_TARGET,
                    WalkTarget::new(
                        Arc::new(EntityTracker::new(vehicle, false)),
                        speed_modifier,
                        1,
                    ),
                );
            }
            true
        },
    )
}

fn start_riding(tick: &BrainTick<'_>, vehicle: &Arc<dyn EntityBase>) {
    let Some(body) = tick.world.get_entity_by_id(tick.mob.get_entity().entity_id) else {
        return;
    };
    vehicle
        .get_entity()
        .add_passenger(Arc::clone(vehicle), body);
}
