use pumpkin_data::{BlockDirection, entity::EntityPose};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use super::MinecartEntity;
use crate::entity::EntityBase;
use crate::entity::dismount_helper::{
    ExitPose, block_floor_height, exceptional_exit, is_block_floor_valid, offsets_for_direction,
    passenger_fits, passenger_poses, try_exit_pose,
};
use crate::world::World;

const MAX_DISMOUNT_WIDTH: f32 = 1.0;

impl MinecartEntity {
    pub(super) fn get_dismount_location_for_passenger(
        &self,
        passenger: &dyn EntityBase,
        still_current: &dyn Fn() -> bool,
    ) -> Option<Vector3<f64>> {
        self.search_dismount_location(passenger, still_current)
            .or_else(|| {
                if !still_current() {
                    return None;
                }
                self.search_dismount_location(passenger, still_current)
            })
    }

    fn search_dismount_location(
        &self,
        passenger: &dyn EntityBase,
        still_current: &dyn Fn() -> bool,
    ) -> Option<Vector3<f64>> {
        let minecart = self.get_entity();
        if let Some(exit) = exceptional_exit(minecart, passenger) {
            return Some(exit);
        }

        let rider = passenger.get_entity();
        let world = rider.world.load();
        let position = minecart.pos.load();
        let block_pos = minecart.block_pos.load();
        let motion_epoch = minecart.vehicle_motion_epoch();
        let direction = self.motion_direction();
        let offsets = offsets_for_direction(direction);
        let can_use_search = || {
            still_current()
                && minecart.pos.load() == position
                && minecart.vehicle_motion_epoch() == motion_epoch
                && !minecart.is_removed()
                && self.motion_direction() == direction
        };
        let poses = passenger_poses(passenger);
        'poses: for &pose in poses {
            let height_offsets: &[i32] = if pose == EntityPose::Swimming {
                &[0, 1]
            } else {
                &[0, 1, -1]
            };
            for &height in height_offsets {
                for offset in offsets {
                    let target_block = block_pos.offset(offset.add_raw(0, height, 0));
                    let floor = block_floor_height(&world, &target_block);
                    if !is_block_floor_valid(floor) {
                        continue;
                    }
                    let target = target_block.0.to_f64().add_raw(0.5, floor, 0.5);
                    match try_exit_pose(rider, pose, &can_use_search, || {
                        block_floor_height(&world, &target_block).to_bits() == floor.to_bits()
                            && passenger_fits(&world, passenger, target, pose, MAX_DISMOUNT_WIDTH)
                    }) {
                        ExitPose::Applied => return Some(target),
                        ExitPose::Obstructed => {}
                        ExitPose::Vetoed => continue 'poses,
                        ExitPose::Superseded => return None,
                    }
                }
            }
        }

        let vehicle_top = minecart.bounding_box.load().max.y;
        let ceiling_start = BlockPos::new(block_pos.0.x, vehicle_top.floor() as i32, block_pos.0.z);
        let fallback = Vector3::new(position.x, vehicle_top, position.z);
        for &pose in poses {
            let fits_under_ceiling = || {
                let height = f64::from(rider.get_dimensions(pose).height);
                let blocks = (vehicle_top - f64::from(ceiling_start.0.y) + height).ceil() as i32;
                vehicle_top + height <= find_ceiling(&world, ceiling_start, blocks)
            };
            match try_exit_pose(rider, pose, &can_use_search, fits_under_ceiling) {
                ExitPose::Applied => return Some(fallback),
                ExitPose::Obstructed | ExitPose::Vetoed => {}
                ExitPose::Superseded => return None,
            }
        }
        can_use_search().then_some(fallback)
    }
    fn motion_direction(&self) -> BlockDirection {
        let direction =
            BlockDirection::from_cardinal_direction(self.get_entity().get_horizontal_facing());
        if self
            .flipped_rotation
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            direction.opposite().rotate_clockwise()
        } else {
            direction.rotate_clockwise()
        }
    }
}

fn find_ceiling(world: &World, base: BlockPos, blocks: i32) -> f64 {
    (0..blocks)
        .find_map(|dy| {
            let pos = base.offset(Vector3::new(0, dy, 0));
            world
                .get_block_state(&pos)
                .get_block_collision_shapes_at(&pos)
                .map(|shape| shape.min.y)
                .reduce(f64::min)
                .map(|surface| f64::from(pos.0.y) + surface)
        })
        .unwrap_or(f64::INFINITY)
}
