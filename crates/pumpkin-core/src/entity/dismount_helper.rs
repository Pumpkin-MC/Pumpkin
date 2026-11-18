use pumpkin_data::block_properties::{CampfireLikeProperties, TrapdoorProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::world::World;

const DISMOUNT_EPSILON: f64 = 1.0E-5f32 as f64;
const MAX_FREE_POSITION_DIMENSION: f32 = 4.0;

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

pub(super) fn can_dismount_passenger(
    world: &World,
    passenger: &dyn EntityBase,
    bounds: BoundingBox,
) -> bool {
    world.get_block_collisions(bounds, passenger).0.is_empty()
        && is_within_border(world, bounds, DISMOUNT_EPSILON)
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

/// Vanilla `LivingEntity.dismountVehicle` when the vehicle was removed or is in a portal.
pub(super) fn find_free_dismount_position(
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
