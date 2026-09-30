use std::sync::Arc;

use crate::entity::EntityBase;

use super::super::BrainTick;
use super::super::memory::types;
use super::one_shot::OneShot;
use super::utils::is_alive;

#[must_use]
pub fn dismount_or_skip_mounting(
    max_walk_dist_to_ride_target: i32,
    dont_ride_if: impl Fn(&BrainTick<'_>, &Arc<dyn EntityBase>) -> bool + Send + Sync + 'static,
) -> OneShot {
    let max_dist_squared = f64::from(max_walk_dist_to_ride_target * max_walk_dist_to_ride_target);
    OneShot::with_required(
        "DismountOrSkipMounting",
        Vec::new(),
        vec![types::RIDE_TARGET.id()],
        move |tick| {
            let current_vehicle = tick.mob.get_entity().get_vehicle();
            let target_vehicle = tick.brain.get(types::RIDE_TARGET).cloned();
            let Some(vehicle) = current_vehicle.clone().or(target_vehicle) else {
                return false;
            };

            let body_pos = tick.mob.get_entity().pos.load();
            let vehicle_valid = is_alive(vehicle.as_ref())
                && vehicle
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&body_pos)
                    < max_dist_squared
                && Arc::ptr_eq(&vehicle.get_entity().world.load_full(), tick.world);
            if vehicle_valid && !dont_ride_if(tick, &vehicle) {
                return false;
            }

            if let Some(current) = current_vehicle {
                current
                    .get_entity()
                    .remove_passenger(tick.mob.get_entity().entity_id);
            }
            tick.brain.erase(types::RIDE_TARGET.id());
            true
        },
    )
}
