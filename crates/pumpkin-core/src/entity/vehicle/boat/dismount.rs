use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    cos,
    position::BlockPos,
    sin,
    vector3::Vector3,
};

use super::BoatEntity;
use crate::entity::EntityBase;
use crate::world::World;

const DISMOUNT_EPSILON: f64 = 1.0e-5f32 as f64;

impl BoatEntity {
    pub(in crate::entity) fn get_dismount_location_for_passenger(
        &self,
        passenger: &dyn EntityBase,
    ) -> Vector3<f64> {
        let entity = &self.vehicle.entity;
        let passenger_entity = passenger.get_entity();
        let pos = entity.pos.load();
        let top = entity.bounding_box.load().max.y;
        let fallback = Vector3::new(pos.x, top, pos.z);
        let direction = get_collision_horizontal_escape_vector(
            f64::from(entity.width() * std::f32::consts::SQRT_2),
            f64::from(passenger_entity.width()),
            passenger_entity.yaw.load(),
        );
        let target = fallback + direction;
        let target_pos = BlockPos::floored_v(target);
        let world = entity.world.load();
        let (fluid, _) = world.get_fluid_and_fluid_state(&target_pos.down());
        if fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            return fallback;
        }

        let targets = [target_pos, target_pos.down()].map(|block_pos| {
            let floor = world.get_dismount_height(&block_pos);
            (floor.is_finite() && floor < 1.0).then_some(Vector3::new(
                target.x,
                f64::from(block_pos.0.y) + floor,
                target.z,
            ))
        });
        let is_player = passenger.get_player().is_some();
        let poses = [
            EntityPose::Standing,
            EntityPose::Crouching,
            EntityPose::Swimming,
        ];
        for pose in poses.into_iter().take(if is_player { 3 } else { 1 }) {
            let dimensions = if is_player {
                passenger_entity.get_dimensions(pose)
            } else {
                passenger_entity.entity_dimension.load()
            };
            for target in targets.into_iter().flatten() {
                if !can_dismount_to(&world, passenger, target, &dimensions) {
                    continue;
                }
                if is_player {
                    passenger_entity.set_pose(pose);
                    if passenger_entity.pose.load() != pose {
                        continue;
                    }
                }
                return target;
            }
        }
        fallback
    }
}

fn get_collision_horizontal_escape_vector(
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

fn can_dismount_to(
    world: &World,
    passenger: &dyn EntityBase,
    location: Vector3<f64>,
    dimensions: &EntityDimensions,
) -> bool {
    let bounds = BoundingBox::new_from_pos(location.x, location.y, location.z, dimensions);
    if !world.get_block_collisions(bounds, passenger).0.is_empty() {
        return false;
    }
    let border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    border.contains(bounds.min.x, bounds.min.z)
        && border.contains(
            bounds.max.x - DISMOUNT_EPSILON,
            bounds.max.z - DISMOUNT_EPSILON,
        )
}
