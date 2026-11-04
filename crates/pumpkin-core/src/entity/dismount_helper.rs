use pumpkin_data::block_properties::{CampfireLikeProperties, TrapdoorProperties};
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{Block as BlockTag, Taggable};
use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

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

fn is_block_floor_valid(block_floor_height: f64) -> bool {
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

fn block_floor_height(world: &World, pos: &BlockPos) -> f64 {
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
