use pumpkin_data::dimension::Dimension;
use pumpkin_data::poi_type::PoiType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::poi::Occupancy;

use super::super::OneShot;
use crate::entity::ai::brain::memory::{
    GlobalPos, MemoryModuleId, MemoryStatus, types, walk_target::WalkTarget,
};

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 7] = [
    (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
    (types::HOME.id(), MemoryStatus::Registered),
    (types::HIDING_PLACE.id(), MemoryStatus::Registered),
    (types::PATH.id(), MemoryStatus::Registered),
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
    (types::BREED_TARGET.id(), MemoryStatus::Registered),
    (types::INTERACTION_TARGET.id(), MemoryStatus::Registered),
];

fn closer_to_center_than(pos: BlockPos, body_pos: Vector3<f64>, distance: i32) -> bool {
    pos.dist_to_center_sqr(body_pos.x, body_pos.y, body_pos.z)
        < f64::from(distance) * f64::from(distance)
}

#[must_use]
pub fn create(radius: i32, speed_modifier: f32, close_enough_dist: i32) -> OneShot {
    OneShot::new(
        "LocateHidingPlace",
        ENTRY_CONDITIONS.to_vec(),
        move |tick| {
            let body_pos = tick.mob.get_entity().pos.load();
            let body_block_pos = tick.mob.get_entity().block_pos.load();
            let closest = {
                let mut pois = tick
                    .world
                    .portal_poi
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                pois.find_closest(
                    &|poi_type| poi_type == PoiType::Home,
                    &|_| true,
                    body_block_pos,
                    close_enough_dist + 1,
                    Occupancy::Any,
                )
            };
            let mut random = tick.mob.get_random();
            let pos = closest
                .filter(|pos| closer_to_center_than(*pos, body_pos, close_enough_dist))
                .or_else(|| {
                    let mut pois = tick
                        .world
                        .portal_poi
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    pois.get_random(
                        &|poi_type| poi_type == PoiType::Home,
                        &|_| true,
                        Occupancy::Any,
                        body_block_pos,
                        radius,
                        &mut random,
                    )
                })
                .or_else(|| tick.state.get(types::HOME).map(|home| home.pos));
            if let Some(pos) = pos {
                tick.state.erase(types::PATH.id());
                tick.state.erase(types::LOOK_TARGET.id());
                tick.state.erase(types::BREED_TARGET.id());
                tick.state.erase(types::INTERACTION_TARGET.id());
                if let Some(dimension) = Dimension::from_name(tick.world.dimension.minecraft_name) {
                    tick.state
                        .set(types::HIDING_PLACE, GlobalPos::new(dimension, pos));
                }
                if !closer_to_center_than(pos, body_pos, close_enough_dist) {
                    tick.state.set(
                        types::WALK_TARGET,
                        WalkTarget::from_block_pos(pos, speed_modifier, close_enough_dist),
                    );
                }
            }
            true
        },
    )
}
