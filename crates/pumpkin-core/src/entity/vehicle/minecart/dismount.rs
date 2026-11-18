use std::sync::atomic::Ordering;

use pumpkin_data::{
    BlockDirection,
    entity::EntityPose,
    tag::{self, Taggable},
};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use super::MinecartEntity;
use crate::entity::dismount_helper::{
    block_floor_height, can_dismount_passenger, find_free_dismount_position, is_block_floor_valid,
    offsets_for_direction,
};
use crate::entity::{Entity, EntityBase};
use crate::world::World;

impl MinecartEntity {
    pub(in crate::entity) fn get_dismount_location_for_passenger(
        minecart: &Entity,
        passenger: &dyn EntityBase,
    ) -> Vector3<f64> {
        let rider = passenger.get_entity();
        let rider_pos = rider.pos.load();
        if rider.is_removed() || passenger.get_living_entity().is_none() {
            return rider_pos;
        }
        let world = rider.world.load();
        let minecart_pos = minecart.pos.load();
        let block_pos = minecart.block_pos.load();
        if minecart.is_removed()
            || world
                .get_block(&block_pos)
                .has_tag(&tag::Block::MINECRAFT_PORTALS)
        {
            let target = Vector3::new(rider_pos.x, rider_pos.y.max(minecart_pos.y), rider_pos.z);
            return find_free_dismount_position(&world, passenger, target);
        }

        let flipped = world
            .get_entity_by_id(minecart.entity_id)
            .as_deref()
            .and_then(|entity| entity.cast_any().downcast_ref::<Self>())
            .is_some_and(|cart| cart.flipped_rotation.load(Ordering::Relaxed));
        let offsets = offsets_for_direction(motion_direction(minecart.yaw.load(), flipped));
        let poses: &[EntityPose] = if passenger.get_player().is_some() {
            &[
                EntityPose::Standing,
                EntityPose::Crouching,
                EntityPose::Swimming,
            ]
        } else {
            &[EntityPose::Standing]
        };
        for &pose in poses {
            let mut dimensions = rider.get_dimensions(pose);
            dimensions.width = dimensions.width.min(1.0);
            let height_offsets: &[i32] = if pose == EntityPose::Swimming {
                &[0, 1]
            } else {
                &[0, 1, -1]
            };
            'heights: for &height in height_offsets {
                for offset in offsets {
                    let target_block = block_pos.offset(offset.add_raw(0, height, 0));
                    let floor = block_floor_height(&world, &target_block);
                    if !is_block_floor_valid(floor) {
                        continue;
                    }
                    let target = target_block.0.to_f64().add_raw(0.5, floor, 0.5);
                    let bounds =
                        BoundingBox::new_from_pos(target.x, target.y, target.z, &dimensions);
                    if can_dismount_passenger(&world, passenger, bounds) {
                        rider.set_pose(pose);
                        if rider.pose.load() == pose {
                            return target;
                        }
                        break 'heights;
                    }
                }
            }
        }

        let vehicle_top = minecart.bounding_box.load().max.y;
        let ceiling_start = BlockPos::new(block_pos.0.x, vehicle_top.floor() as i32, block_pos.0.z);
        for &pose in poses {
            let height = f64::from(rider.get_dimensions(pose).height);
            let blocks = (vehicle_top - f64::from(ceiling_start.0.y) + height).ceil() as i32;
            if vehicle_top + height <= find_ceiling(&world, ceiling_start, blocks) {
                rider.set_pose(pose);
                if rider.pose.load() == pose {
                    break;
                }
            }
        }
        Vector3::new(minecart_pos.x, vehicle_top, minecart_pos.z)
    }
}

/// Legacy minecart yaw describes the model axis, a quarter-turn from its motion.
fn motion_direction(yaw: f32, flipped: bool) -> BlockDirection {
    let facing = match ((f64::from(yaw) / 90.0 + 0.5).floor() as i32) & 3 {
        0 => BlockDirection::South,
        1 => BlockDirection::West,
        2 => BlockDirection::North,
        _ => BlockDirection::East,
    };
    let facing = if flipped { facing.opposite() } else { facing };
    facing.rotate_clockwise()
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
