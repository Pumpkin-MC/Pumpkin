use std::f64::consts::FRAC_PI_2;

use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::poi::SectionPos;

use super::wander_around::WanderAroundGoal;
use crate::entity::ai::brain::behavior::poi_walk::village_bound_random_stroll::{
    find_section_closest_to_village, section_center,
};
use crate::entity::ai::util::default_random_pos;
use crate::entity::mob::Mob;

const MAX_XZ_DIST: i32 = 10;
const MAX_Y_DIST: i32 = 7;
const INTERVAL: i32 = 10;
const SECTION_SEARCH_RADIUS: i32 = 2;

#[must_use]
pub const fn move_back_to_village(speed: f64) -> WanderAroundGoal {
    WanderAroundGoal::with_interval(speed, INTERVAL)
        .with_check_no_action_time(false)
        .with_start_condition(is_outside_village)
        .with_position(get_position, 0.0)
}

fn is_outside_village(mob: &dyn Mob) -> bool {
    let world = mob.get_entity().world.load();
    !world.is_village(&mob.get_entity().block_pos.load())
}

fn get_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
    let world = mob.get_entity().world.load();
    let section_pos = SectionPos::of(&mob.get_entity().block_pos.load());
    let optimal_section_pos =
        find_section_closest_to_village(&world, section_pos, SECTION_SEARCH_RADIUS);
    if optimal_section_pos == section_pos {
        return None;
    }
    default_random_pos::get_pos_towards(
        mob,
        MAX_XZ_DIST,
        MAX_Y_DIST,
        section_center(optimal_section_pos).to_f64(),
        FRAC_PI_2,
    )
}
