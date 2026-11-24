use pumpkin_data::block_properties::{CampfireLikeProperties, TrapdoorProperties};
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::tag::{self, Block as BlockTag, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockState, HorizontalFacingExt};
use pumpkin_util::Hand;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{cos, sin};

use crate::entity::{Entity, EntityBase};
use crate::world::World;

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

pub(crate) fn is_block_floor_valid(block_floor_height: f64) -> bool {
    block_floor_height.is_finite() && block_floor_height < 1.0
}

fn can_dismount_to(world: &World, bounding_box: &BoundingBox) -> bool {
    if !world.is_space_empty(*bounding_box) {
        return false;
    }

    let border = world
        .worldborder
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    border.contains(bounding_box.min.x, bounding_box.min.z)
        && border.contains(bounding_box.max.x - 1.0E-5, bounding_box.max.z - 1.0E-5)
}

/// Vanilla `getBlockFloorHeight` over `DismountHelper.nonClimbableShape`.
pub(crate) fn block_floor_height(world: &World, pos: &BlockPos) -> f64 {
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

/// Vanilla `getDismountPoses`.
fn dismount_poses(passenger: &dyn EntityBase) -> &'static [EntityPose] {
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

/// Vanilla `getLocalBoundsForPose` moved to `location`.
fn pose_box(passenger: &dyn EntityBase, pose: EntityPose, location: Vector3<f64>) -> BoundingBox {
    let dimensions = passenger.get_entity().get_dimensions(pose);
    BoundingBox::new_from_pos(location.x, location.y, location.z, &dimensions)
}

/// Vanilla `DismountHelper.canDismountTo(level, passenger, box)`.
fn passenger_fits(world: &World, passenger: &dyn EntityBase, bounding_box: BoundingBox) -> bool {
    world
        .get_block_collisions(bounding_box, passenger)
        .0
        .is_empty()
        && can_dismount_to(world, &bounding_box)
}

/// The first dismount pose of `passenger` that fits at `location`, which it then takes.
fn dismount_pose_fits(world: &World, passenger: &dyn EntityBase, location: Vector3<f64>) -> bool {
    dismount_poses(passenger).iter().any(|&pose| {
        let fits = passenger_fits(world, passenger, pose_box(passenger, pose, location));
        if fits {
            passenger.get_entity().set_pose(pose);
        }
        fits
    })
}

/// Vanilla `Vec3.upFromBottomCenterOf`.
fn up_from_bottom_center(pos: BlockPos, height: f64) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y) + height,
        f64::from(pos.0.z) + 0.5,
    )
}

/// Vanilla `DismountHelper.offsetsForDirection`: sides first, then the diagonals, back and
/// front.
fn offsets_for_direction(forward: BlockDirection) -> [(i32, i32); 8] {
    let step = |direction: BlockDirection| {
        let offset = direction.to_offset();
        (offset.x, offset.z)
    };
    let right = step(forward.rotate_clockwise());
    let left = step(forward.rotate_clockwise().opposite());
    let back = step(forward.opposite());
    let forward = step(forward);
    [
        right,
        left,
        (back.0 + right.0, back.1 + right.1),
        (back.0 + left.0, back.1 + left.1),
        (forward.0 + right.0, forward.1 + right.1),
        (forward.0 + left.0, forward.1 + left.1),
        back,
        forward,
    ]
}

/// Vanilla `Entity.getCollisionHorizontalEscapeVector`: just outside both boxes, towards
/// `yaw`.
fn escape_vector(collider_width: f64, colliding_width: f64, yaw: f32) -> Vector3<f64> {
    let distance = (collider_width + colliding_width + f64::from(1.0e-5f32)) / 2.0;
    let radians = yaw * (std::f64::consts::PI / 180.0) as f32;
    let x = -sin(radians);
    let z = cos(radians);
    let scale = f64::from(x.abs().max(z.abs()));
    Vector3::new(
        f64::from(x) * distance / scale,
        0.0,
        f64::from(z) * distance / scale,
    )
}

/// Vanilla `Entity.getDismountLocationForPassenger`: on top of the vehicle.
#[must_use]
pub fn vehicle_top(vehicle: &Entity) -> Vector3<f64> {
    let pos = vehicle.pos.load();
    Vector3::new(pos.x, vehicle.bounding_box.load().max.y, pos.z)
}

/// Vanilla `Animal.getDismountLocationForPassenger`: a free block around the animal.
pub fn animal_dismount_location(vehicle: &Entity, passenger: &dyn EntityBase) -> Vector3<f64> {
    let world = vehicle.world.load();
    let forward = vehicle.get_horizontal_facing().to_block_direction();
    let base = vehicle.block_pos.load();
    for &pose in dismount_poses(passenger) {
        for (x, z) in offsets_for_direction(forward) {
            let target = BlockPos::new(base.0.x + x, base.0.y, base.0.z + z);
            let floor = world.get_dismount_height(&target);
            if !is_block_floor_valid(floor) {
                continue;
            }
            let location = up_from_bottom_center(target, floor);
            if passenger_fits(&world, passenger, pose_box(passenger, pose, location)) {
                passenger.get_entity().set_pose(pose);
                return location;
            }
        }
    }
    vehicle_top(vehicle)
}

/// Vanilla `AbstractMinecart.POSE_DISMOUNT_HEIGHTS`.
const fn pose_dismount_heights(pose: EntityPose) -> &'static [i32] {
    match pose {
        EntityPose::Swimming => &[0, 1],
        _ => &[0, 1, -1],
    }
}

/// Vanilla `DismountHelper.findCeilingFrom` over the plain collision shapes.
fn ceiling_from(world: &World, pos: BlockPos, blocks: i32) -> f64 {
    (0..blocks)
        .find_map(|y| {
            let cursor = BlockPos::new(pos.0.x, pos.0.y + y, pos.0.z);
            world
                .get_block_state(&cursor)
                .get_block_collision_shapes_at(&cursor)
                .map(|shape| shape.min.y)
                .reduce(f64::min)
                .map(|bottom| f64::from(pos.0.y + y) + bottom)
        })
        .unwrap_or(f64::INFINITY)
}

/// Vanilla `AbstractMinecart.getDismountLocationForPassenger`: beside the cart, across its
/// `forward` motion, else on top in the first pose that fits under the ceiling.
pub fn minecart_dismount_location(
    vehicle: &Entity,
    forward: BlockDirection,
    passenger: &dyn EntityBase,
) -> Vector3<f64> {
    let world = vehicle.world.load();
    let base = vehicle.block_pos.load();
    let offsets = offsets_for_direction(forward);
    for &pose in dismount_poses(passenger) {
        let dimensions = passenger.get_entity().get_dimensions(pose);
        let reach = f64::from(dimensions.width.min(1.0) / 2.0);
        for &y in pose_dismount_heights(pose) {
            for (x, z) in offsets {
                let target = BlockPos::new(base.0.x + x, base.0.y + y, base.0.z + z);
                let floor = block_floor_height(&world, &target);
                if !is_block_floor_valid(floor) {
                    continue;
                }
                let location = up_from_bottom_center(target, floor);
                let reach_box = BoundingBox::new(
                    location.add_raw(-reach, 0.0, -reach),
                    location.add_raw(reach, f64::from(dimensions.height), reach),
                );
                if passenger_fits(&world, passenger, reach_box) {
                    passenger.get_entity().set_pose(pose);
                    return location;
                }
            }
        }
    }

    let vehicle_top_y = vehicle.bounding_box.load().max.y;
    let target = BlockPos::new(base.0.x, vehicle_top_y.floor() as i32, base.0.z);
    for &pose in dismount_poses(passenger) {
        let height = f64::from(passenger.get_entity().get_dimensions(pose).height);
        let coverage = (vehicle_top_y - f64::from(target.0.y) + height).ceil() as i32;
        if vehicle_top_y + height <= ceiling_from(&world, target, coverage) {
            passenger.get_entity().set_pose(pose);
            break;
        }
    }
    vehicle_top(vehicle)
}

/// Vanilla `AbstractBoat.getDismountLocationForPassenger`: beside the boat towards where the
/// passenger looks, unless that is over water.
pub fn boat_dismount_location(vehicle: &Entity, passenger: &dyn EntityBase) -> Vector3<f64> {
    let world = vehicle.world.load();
    let direction = escape_vector(
        f64::from(vehicle.width() * std::f32::consts::SQRT_2),
        f64::from(passenger.get_entity().width()),
        passenger.get_entity().yaw.load(),
    );
    let pos = vehicle.pos.load();
    let (x, z) = (pos.x + direction.x, pos.z + direction.z);
    let target = BlockPos::floored(x, vehicle.bounding_box.load().max.y, z);
    let below = target.down();
    if !world.fluid_state_has_tag(&below, &tag::Fluid::MINECRAFT_WATER) {
        let targets: Vec<Vector3<f64>> = [target, below]
            .into_iter()
            .filter_map(|block| {
                let floor = world.get_dismount_height(&block);
                is_block_floor_valid(floor)
                    .then(|| Vector3::new(x, f64::from(block.0.y) + floor, z))
            })
            .collect();
        for &pose in dismount_poses(passenger) {
            for &location in &targets {
                if passenger_fits(&world, passenger, pose_box(passenger, pose, location)) {
                    passenger.get_entity().set_pose(pose);
                    return location;
                }
            }
        }
    }
    vehicle_top(vehicle)
}

/// Vanilla `LivingEntity.getMainArm`.
fn main_arm(passenger: &dyn EntityBase) -> Hand {
    if let Some(player) = passenger.get_player() {
        return player.config.load().main_hand;
    }
    if passenger
        .get_mob()
        .is_some_and(|mob| mob.get_mob_entity().is_left_handed())
    {
        Hand::Left
    } else {
        Hand::Right
    }
}

/// Vanilla `AbstractHorse.getDismountLocationForPassenger`: off the passenger's main hand
/// side, else the other side, else where the horse stands.
pub fn horse_dismount_location(vehicle: &Entity, passenger: &dyn EntityBase) -> Vector3<f64> {
    let yaw = vehicle.yaw.load();
    let (width, passenger_width) = (
        f64::from(vehicle.width()),
        f64::from(passenger.get_entity().width()),
    );
    let side = |hand: Hand| {
        if hand == main_arm(passenger) {
            90.0
        } else {
            -90.0
        }
    };
    [side(Hand::Right), side(Hand::Left)]
        .into_iter()
        .find_map(|turn| {
            let direction = escape_vector(width, passenger_width, yaw + turn);
            horse_dismount_in_direction(vehicle, direction, passenger)
        })
        .unwrap_or_else(|| vehicle.pos.load())
}

/// Vanilla `AbstractHorse.getDismountLocationInDirection`: the passenger may step up to
/// `0.75` above the horse's top.
fn horse_dismount_in_direction(
    vehicle: &Entity,
    direction: Vector3<f64>,
    passenger: &dyn EntityBase,
) -> Option<Vector3<f64>> {
    let world = vehicle.world.load();
    let pos = vehicle.pos.load();
    let bounding_box = vehicle.bounding_box.load();
    let (x, z) = (pos.x + direction.x, pos.z + direction.z);
    let jump_limit = bounding_box.max.y + 0.75;
    for &pose in dismount_poses(passenger) {
        let mut target = BlockPos::floored(x, bounding_box.min.y, z);
        loop {
            let floor = world.get_dismount_height(&target);
            let y = f64::from(target.0.y) + floor;
            if y > jump_limit {
                break;
            }
            if is_block_floor_valid(floor) {
                let location = Vector3::new(x, y, z);
                if passenger_fits(&world, passenger, pose_box(passenger, pose, location)) {
                    passenger.get_entity().set_pose(pose);
                    return Some(location);
                }
            }
            target = target.up();
            if f64::from(target.0.y) >= jump_limit {
                break;
            }
        }
    }
    None
}

/// Vanilla `Strider.getDismountLocationForPassenger`: a block out of lava around the
/// passenger's view, from the strider's top down.
pub fn strider_dismount_location(vehicle: &Entity, passenger: &dyn EntityBase) -> Vector3<f64> {
    let world = vehicle.world.load();
    let pos = vehicle.pos.load();
    let bounding_box = vehicle.bounding_box.load();
    let (width, passenger_width) = (
        f64::from(vehicle.width()),
        f64::from(passenger.get_entity().width()),
    );
    let yaw = passenger.get_entity().yaw.load();
    let top = bounding_box.max.y;
    let bottom = bounding_box.min.y - 0.5;
    let mut targets: Vec<BlockPos> = Vec::new();
    for turn in [0.0, -22.5, 22.5, -45.0, 45.0] {
        let direction = escape_vector(width, passenger_width, yaw + turn);
        let mut target = BlockPos::floored(pos.x + direction.x, top, pos.z + direction.z);
        let mut y = top;
        while y > bottom {
            if !targets.contains(&target) {
                targets.push(target);
            }
            target = target.down();
            y -= 1.0;
        }
    }
    for target in targets {
        if world.fluid_state_has_tag(&target, &tag::Fluid::MINECRAFT_LAVA) {
            continue;
        }
        let floor = world.get_dismount_height(&target);
        if is_block_floor_valid(floor) {
            let location = up_from_bottom_center(target, floor);
            if dismount_pose_fits(&world, passenger, location) {
                return location;
            }
        }
    }
    vehicle_top(vehicle)
}
