use pumpkin_data::tag;
use pumpkin_util::math::{cos, position::BlockPos, sin, vector3::Vector3};

use super::BoatEntity;
use crate::entity::dismount_helper::{
    ExitPose, exceptional_exit, is_block_floor_valid, passenger_fits, passenger_poses,
    try_exit_pose,
};
use crate::entity::{Entity, EntityBase};

const ESCAPE_EPSILON: f64 = 1.0E-5f32 as f64;

impl BoatEntity {
    pub(in crate::entity) fn get_dismount_location_for_passenger(
        boat: &Entity,
        passenger: &dyn EntityBase,
        still_current: &dyn Fn() -> bool,
    ) -> Option<Vector3<f64>> {
        if let Some(exit) = exceptional_exit(boat, passenger) {
            return Some(exit);
        }

        let rider = passenger.get_entity();
        let world = rider.world.load();
        let boat_pos = boat.pos.load();
        let boat_box = boat.bounding_box.load();
        let fallback = Vector3::new(boat_pos.x, boat_box.max.y, boat_pos.z);
        let can_use_search =
            || still_current() && boat.pos.load() == boat_pos && !boat.is_removed();
        let distance = (f64::from(boat.width() * std::f32::consts::SQRT_2)
            + f64::from(rider.width())
            + ESCAPE_EPSILON)
            / 2.0;
        let yaw = rider.yaw.load().to_radians();
        let (x, z) = (-sin(yaw), cos(yaw));
        let scale = f64::from(x.abs().max(z.abs()));
        let target = fallback.add_raw(
            f64::from(x) * distance / scale,
            0.0,
            f64::from(z) * distance / scale,
        );
        let target_block = BlockPos::floored_v(target);
        let below = target_block.down();
        let target_is_dry = || !world.fluid_state_has_tag(&below, &tag::Fluid::MINECRAFT_WATER);
        if !target_is_dry() {
            return Some(fallback);
        }

        for &pose in passenger_poses(passenger) {
            for block in [target_block, below] {
                let floor = world.get_dismount_height(&block);
                if !is_block_floor_valid(floor) {
                    continue;
                }
                let exit = Vector3::new(target.x, f64::from(block.0.y) + floor, target.z);
                match try_exit_pose(rider, pose, &can_use_search, || {
                    target_is_dry()
                        && world.get_dismount_height(&block).to_bits() == floor.to_bits()
                        && passenger_fits(&world, passenger, exit, pose, f32::INFINITY)
                }) {
                    ExitPose::Applied => return Some(exit),
                    ExitPose::Obstructed => {}
                    ExitPose::Vetoed => break,
                    ExitPose::Superseded => return None,
                }
            }
        }
        can_use_search().then_some(fallback)
    }
}
