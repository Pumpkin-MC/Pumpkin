use std::sync::Arc;

use pumpkin_data::block_properties::{CampfireLikeProperties, TrapdoorProperties};
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{Entity, EntityBase};
use crate::world::World;

const MAX_FREE_POSITION_DIMENSION: f32 = 4.0;
const BORDER_EPSILON: f64 = 1.0E-5f32 as f64;

pub(super) fn offsets_for_yaw(yaw: f32) -> [Vector3<i32>; 8] {
    let yaw = (yaw % 360.0 + 360.0) % 360.0;
    let direction = if !(45.0..315.0).contains(&yaw) {
        BlockDirection::South
    } else if yaw < 135.0 {
        BlockDirection::West
    } else if yaw < 225.0 {
        BlockDirection::North
    } else {
        BlockDirection::East
    };
    offsets_for_direction(direction)
}

pub(super) fn offsets_for_direction(forward: BlockDirection) -> [Vector3<i32>; 8] {
    let right = forward.rotate_clockwise().to_offset();
    let left = forward.rotate_counter_clockwise().to_offset();
    let back = forward.opposite().to_offset();
    let forward = forward.to_offset();
    [
        right,
        left,
        back + right,
        back + left,
        forward + right,
        forward + left,
        back,
        forward,
    ]
}

pub(super) fn passenger_poses(passenger: &dyn EntityBase) -> &'static [EntityPose] {
    if passenger.get_player().is_some() {
        &[
            EntityPose::Standing,
            EntityPose::Crouching,
            EntityPose::Swimming,
        ]
    } else {
        &[EntityPose::Standing]
    }
}

pub(super) fn passenger_fits(
    world: &World,
    passenger: &dyn EntityBase,
    target: Vector3<f64>,
    pose: EntityPose,
    max_width: f32,
) -> bool {
    let rider = passenger.get_entity();
    let mut dimensions = rider.get_dimensions(pose);
    dimensions.width = dimensions.width.min(max_width);
    let bounds = BoundingBox::new_from_pos(target.x, target.y, target.z, &dimensions);
    world.get_block_collisions(bounds, passenger).0.is_empty()
        && is_within_border(world, bounds, BORDER_EPSILON)
}

pub(super) enum ExitPose {
    Applied,
    Obstructed,
    Vetoed,
    Superseded,
}

/// Recheck the candidate after the pose event and again after outgoing metadata observers.
pub(super) fn try_exit_pose(
    rider: &Entity,
    pose: EntityPose,
    still_current: &dyn Fn() -> bool,
    fits: impl Fn() -> bool,
) -> ExitPose {
    if !still_current() {
        return ExitPose::Superseded;
    }
    if !fits() {
        return ExitPose::Obstructed;
    }
    let previous_pose = rider.pose.load();
    if previous_pose == pose {
        return ExitPose::Applied;
    }

    let has_previous_pose = || still_current() && rider.pose.load() == previous_pose;
    let mut candidate_changed = false;
    if rider.set_pose_if(pose, || {
        if !has_previous_pose() {
            return false;
        }
        candidate_changed = !fits();
        !candidate_changed
    }) {
        if !still_current() || rider.pose.load() != pose {
            ExitPose::Superseded
        } else if !fits() {
            ExitPose::Obstructed
        } else {
            ExitPose::Applied
        }
    } else if !has_previous_pose() {
        ExitPose::Superseded
    } else if candidate_changed {
        ExitPose::Obstructed
    } else {
        ExitPose::Vetoed
    }
}

/// Shared exit for vehicles that use the carrier's standard search order.
pub(super) fn default_vehicle_exit(
    carrier: &Entity,
    passenger: &dyn EntityBase,
    still_current: &dyn Fn() -> bool,
) -> Option<Vector3<f64>> {
    if let Some(exit) = exceptional_exit(carrier, passenger) {
        return still_current().then_some(exit);
    }
    let rider = passenger.get_entity();
    let world = rider.world.load();
    let position = carrier.pos.load();
    let yaw = carrier.yaw.load();
    let top = carrier.bounding_box.load().max.y;
    let block = carrier.block_pos.load();
    let fallback = Vector3::new(position.x, top, position.z);
    let usable = || {
        still_current()
            && !carrier.is_removed()
            && Arc::ptr_eq(&world, &carrier.world.load())
            && carrier.pos.load() == position
            && carrier.yaw.load().to_bits() == yaw.to_bits()
    };
    let below = BlockPos::floored_v(fallback).down();
    let in_water = Fluid::from_state_id(world.get_block_state_id(&below))
        .is_some_and(|fluid| fluid.id == Fluid::WATER.id || fluid.id == Fluid::FLOWING_WATER.id);
    if in_water {
        return usable().then_some(fallback);
    }

    let offsets = offsets_for_yaw(yaw);
    let fits = |pos, pose| passenger_fits(&world, passenger, pos, pose, f32::INFINITY);
    'poses: for &pose in passenger_poses(passenger) {
        let heights: &[i32] = if pose == EntityPose::Swimming {
            &[0, 1]
        } else {
            &[0, 1, -1]
        };
        for &height in heights {
            for offset in offsets {
                let target = block.offset(offset.add_raw(0, height, 0));
                let floor = block_floor_height(&world, &target);
                if !is_block_floor_valid(floor) {
                    continue;
                }
                let pos = target.0.to_f64().add_raw(0.5, floor, 0.5);
                match try_exit_pose(rider, pose, &usable, || {
                    block_floor_height(&world, &target).to_bits() == floor.to_bits()
                        && fits(pos, pose)
                }) {
                    ExitPose::Applied => return Some(pos),
                    ExitPose::Obstructed => {}
                    ExitPose::Vetoed => continue 'poses,
                    ExitPose::Superseded => return None,
                }
            }
        }
    }

    for &pose in passenger_poses(passenger) {
        match try_exit_pose(rider, pose, &usable, || fits(fallback, pose)) {
            ExitPose::Applied => return Some(fallback),
            ExitPose::Obstructed | ExitPose::Vetoed => {}
            ExitPose::Superseded => return None,
        }
    }
    usable().then_some(fallback)
}

/// LivingEntity.dismountVehicle handles removed vehicles and portals before the vehicle's search.
pub(super) fn exceptional_exit(
    vehicle: &Entity,
    passenger: &dyn EntityBase,
) -> Option<Vector3<f64>> {
    let rider = passenger.get_entity();
    let position = rider.pos.load();
    if rider.is_removed() || passenger.get_living_entity().is_none() {
        return Some(position);
    }

    let world = rider.world.load();
    if vehicle.is_removed()
        || world
            .get_block(&vehicle.block_pos.load())
            .has_tag(&BlockTag::MINECRAFT_PORTALS)
    {
        let target = Vector3::new(position.x, position.y.max(vehicle.pos.load().y), position.z);
        return Some(find_free_position(&world, passenger, target));
    }
    None
}

fn find_free_position(
    world: &World,
    passenger: &dyn EntityBase,
    target: Vector3<f64>,
) -> Vector3<f64> {
    let dimensions = passenger.get_entity().entity_dimension.load();
    if dimensions.width > MAX_FREE_POSITION_DIMENSION
        || dimensions.height > MAX_FREE_POSITION_DIMENSION
    {
        return target;
    }
    let width = f64::from(dimensions.width);
    let height = f64::from(dimensions.height);
    let center = target.add_raw(0.0, height / 2.0, 0.0);
    world
        .find_free_position(
            passenger,
            &BoundingBox::of_size(center, width, height, width),
            center,
            Vector3::new(width, height, width),
        )
        .map_or(target, |free| free.add_raw(0.0, -height / 2.0, 0.0))
}

pub fn find_safe_dismount_location(
    entity_type: &'static EntityType,
    world: &World,
    pos: &BlockPos,
    check_dangerous: bool,
) -> Option<Vector3<f64>> {
    if check_dangerous && is_block_dangerous(entity_type, world.get_block_state(pos)) {
        return None;
    }

    let floor_height = block_floor_height(world, pos);
    if !is_block_floor_valid(floor_height) {
        return None;
    }

    if check_dangerous
        && floor_height <= 0.0
        && is_block_dangerous(entity_type, world.get_block_state(&pos.down()))
    {
        return None;
    }

    let position = Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y) + floor_height,
        f64::from(pos.0.z) + 0.5,
    );
    let dimensions = EntityDimensions::new(
        entity_type.dimension[0],
        entity_type.dimension[1],
        entity_type.eye_height,
    );
    let aabb = BoundingBox::new_from_pos(position.x, position.y, position.z, &dimensions);
    if !can_dismount_to(world, &aabb) {
        return None;
    }

    if entity_type == &EntityType::PLAYER {
        let invalid = |pos: &BlockPos| {
            Block::from_state_id(world.get_block_state(pos).id)
                .has_tag(&BlockTag::MINECRAFT_INVALID_SPAWN_INSIDE)
        };
        if invalid(pos) || invalid(&pos.up()) {
            return None;
        }
    }

    Some(position)
}

pub(super) fn is_block_floor_valid(block_floor_height: f64) -> bool {
    block_floor_height.is_finite() && block_floor_height < 1.0
}

fn can_dismount_to(world: &World, bounding_box: &BoundingBox) -> bool {
    if !world.is_space_empty(*bounding_box) {
        return false;
    }

    is_within_border(world, *bounding_box, 1.0E-5)
}

fn is_within_border(world: &World, bounds: BoundingBox, epsilon: f64) -> bool {
    let border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    border.contains(bounds.min.x, bounds.min.z)
        && border.contains(bounds.max.x - epsilon, bounds.max.z - epsilon)
}

pub(super) fn block_floor_height(world: &World, pos: &BlockPos) -> f64 {
    if let Some(shape_max) = non_climbable_shape_max_y(world, pos) {
        return shape_max;
    }

    match non_climbable_shape_max_y(world, &pos.down()) {
        Some(below_max) if below_max >= 1.0 => below_max - 1.0,
        _ => f64::NEG_INFINITY,
    }
}

fn non_climbable_shape_max_y(world: &World, pos: &BlockPos) -> Option<f64> {
    let state = world.get_block_state(pos);
    let block = Block::from_state_id(state.id);
    if block.has_tag(&BlockTag::MINECRAFT_CLIMBABLE) {
        return None;
    }
    if block.has_tag(&BlockTag::MINECRAFT_TRAPDOORS)
        && TrapdoorProperties::from_state_id(state.id).open
    {
        return None;
    }

    state
        .get_block_collision_shapes_at(pos)
        .map(|shape| shape.max.y)
        .reduce(f64::max)
}

fn is_block_dangerous(entity_type: &'static EntityType, state: &BlockState) -> bool {
    let block = Block::from_state_id(state.id);
    if block.has_tag(entity_type.immune_to) {
        return false;
    }
    if !entity_type.fire_immune && is_burning_block(block, state) {
        return true;
    }

    block == &Block::WITHER_ROSE
        || block == &Block::SWEET_BERRY_BUSH
        || block == &Block::CACTUS
        || block == &Block::POWDER_SNOW
}

fn is_burning_block(block: &Block, state: &BlockState) -> bool {
    block.has_tag(&BlockTag::MINECRAFT_FIRE)
        || block == &Block::LAVA
        || block == &Block::MAGMA_BLOCK
        || (block.has_tag(&BlockTag::MINECRAFT_CAMPFIRES)
            && CampfireLikeProperties::from_state_id(state.id).lit)
        || block == &Block::LAVA_CAULDRON
}
