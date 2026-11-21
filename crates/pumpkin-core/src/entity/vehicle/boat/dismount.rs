use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{
    boundingbox::BoundingBox, cos, position::BlockPos, sin, vector3::Vector3,
};

use super::BoatEntity;
use crate::entity::dismount_helper::{can_dismount_passenger, find_free_dismount_position};
use crate::entity::{Entity, EntityBase};

const DISMOUNT_EPSILON: f64 = 1.0e-5f32 as f64;

impl BoatEntity {
    pub(in crate::entity) fn get_dismount_location_for_passenger(
        boat: &Entity,
        passenger: &dyn EntityBase,
    ) -> Vector3<f64> {
        let rider = passenger.get_entity();
        let rider_pos = rider.pos.load();
        if rider.is_removed() || passenger.get_living_entity().is_none() {
            return rider_pos;
        }

        let boat_pos = boat.pos.load();
        let world = rider.world.load();
        if boat.is_removed()
            || world
                .get_block(&boat.block_pos.load())
                .has_tag(&tag::Block::MINECRAFT_PORTALS)
        {
            let target = Vector3::new(rider_pos.x, rider_pos.y.max(boat_pos.y), rider_pos.z);
            return find_free_dismount_position(&world, passenger, target);
        }

        let fallback = Vector3::new(boat_pos.x, boat.bounding_box.load().max.y, boat_pos.z);
        let direction = collision_horizontal_escape_vector(
            f64::from(boat.width() * std::f32::consts::SQRT_2),
            f64::from(rider.width()),
            rider.yaw.load(),
        );
        let target = fallback + direction;
        let target_block = BlockPos::floored_v(target);
        let below = target_block.down();
        let (fluid, _) = world.get_fluid_and_fluid_state(&below);
        if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            return fallback;
        }

        let targets = [target_block, below].map(|block| {
            let floor = world.get_dismount_height(&block);
            (floor.is_finite() && floor < 1.0).then_some(Vector3::new(
                target.x,
                f64::from(block.0.y) + floor,
                target.z,
            ))
        });
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
            let dimensions = rider.get_dimensions(pose);
            for target in targets.into_iter().flatten() {
                let bounds = BoundingBox::new_from_pos(target.x, target.y, target.z, &dimensions);
                if can_dismount_passenger(&world, passenger, bounds) {
                    rider.set_pose(pose);
                    if rider.pose.load() == pose {
                        return target;
                    }
                    // A plugin can reject this pose at either candidate height.
                    break;
                }
            }
        }
        fallback
    }
}

fn collision_horizontal_escape_vector(
    collider_width: f64,
    colliding_width: f64,
    direction_degrees: f32,
) -> Vector3<f64> {
    let distance = (collider_width + colliding_width + DISMOUNT_EPSILON) / 2.0;
    let direction = direction_degrees.to_radians();
    let x = -sin(direction);
    let z = cos(direction);
    let scale = f64::from(x.abs().max(z.abs()));
    Vector3::new(
        f64::from(x) * distance / scale,
        0.0,
        f64::from(z) * distance / scale,
    )
}
