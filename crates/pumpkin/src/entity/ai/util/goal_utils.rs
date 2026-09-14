use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::mob::Mob;
use crate::world::World;

#[must_use]
pub fn mob_restricted(mob: &dyn Mob, horizontal_dist: f64) -> bool {
    let mob_entity = mob.get_mob_entity();
    if !mob_entity.has_position_target() {
        return false;
    }
    let home = mob_entity.position_target.load();
    let radius = f64::from(
        mob_entity
            .position_target_range
            .load(std::sync::atomic::Ordering::Relaxed),
    );
    let limit = radius + horizontal_dist + 1.0;
    home.to_centered_f64()
        .squared_distance_to_vec(&mob_entity.living_entity.entity.pos.load())
        < limit * limit
}

#[must_use]
pub const fn is_outside_limits(pos: &BlockPos, world: &World) -> bool {
    pos.0.y < world.dimension.min_y || pos.0.y >= world.dimension.min_y + world.dimension.height
}

#[must_use]
pub fn is_restricted(restrict: bool, mob: &dyn Mob, pos: &BlockPos) -> bool {
    restrict && !mob.get_mob_entity().is_in_position_target_range_pos(pos)
}

/// Nothing solid to stand on.
#[must_use]
pub fn is_not_stable(world: &World, pos: &BlockPos) -> bool {
    !world.get_block_state(&pos.down()).is_solid()
}

#[must_use]
pub fn is_water(world: &World, pos: &BlockPos) -> bool {
    let (_, state_id) = world.get_block_and_state_id(pos);
    if state_id.to_state().is_waterlogged() {
        return true;
    }
    pumpkin_data::fluid::Fluid::from_state_id(state_id)
        .is_some_and(|fluid| fluid.has_tag(&tag::Fluid::MINECRAFT_WATER))
}

/// Takes the navigation lock, so callers must not already hold it.
#[must_use]
pub fn has_malus(mob: &dyn Mob, context: &mut PathfindingContext, pos: &BlockPos) -> bool {
    let path_type = context.get_path_type_from_state(Vector3::new(pos.0.x, pos.0.y, pos.0.z));
    let malus = mob
        .get_mob_entity()
        .navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_pathfinding_malus(path_type);
    malus != 0.0
}

#[must_use]
pub fn is_solid(world: &World, pos: &BlockPos) -> bool {
    world.get_block_state(pos).is_solid()
}
