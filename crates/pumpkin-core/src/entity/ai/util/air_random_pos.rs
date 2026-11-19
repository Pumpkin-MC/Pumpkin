use pumpkin_util::math::vector3::Vector3;

use super::air_and_water_random_pos;
use crate::entity::mob::Mob;

/// Picks a spot in open air, outside water, heading towards `target`.
#[must_use]
pub fn get_pos_towards(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
    flying_height: i32,
    target: Vector3<f64>,
    max_xz_radians_difference: f64,
) -> Option<Vector3<f64>> {
    let delta = target - mob.get_entity().pos.load();
    air_and_water_random_pos::find_pos(
        mob,
        horizontal_dist,
        vertical_dist,
        flying_height,
        [delta.x, delta.z],
        max_xz_radians_difference,
        false,
    )
}
