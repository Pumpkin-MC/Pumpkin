use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub mod go_to_closest_village;
pub mod inside_brownian_walk;
pub mod release_poi;
pub mod set_closest_home_as_walk_target;
pub mod set_walk_target_from_block_memory;
pub mod stroll_around_poi;
pub mod stroll_to_poi;
pub mod stroll_to_poi_list;
pub mod village_bound_random_stroll;

fn closer_to_center_than(pos: BlockPos, entity_pos: Vector3<f64>, distance: f64) -> bool {
    pos.dist_to_center_sqr(entity_pos.x, entity_pos.y, entity_pos.z) < distance * distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closer_to_center_than_is_strict() {
        let pos = BlockPos::new(0, 64, 0);
        assert!(closer_to_center_than(
            pos,
            Vector3::new(0.5, 64.5, 0.5),
            1.0
        ));
        assert!(closer_to_center_than(
            pos,
            Vector3::new(1.4, 64.5, 0.5),
            1.0
        ));
        assert!(!closer_to_center_than(
            pos,
            Vector3::new(1.5, 64.5, 0.5),
            1.0
        ));
        assert!(!closer_to_center_than(
            pos,
            Vector3::new(2.0, 64.5, 0.5),
            1.0
        ));
    }
}
