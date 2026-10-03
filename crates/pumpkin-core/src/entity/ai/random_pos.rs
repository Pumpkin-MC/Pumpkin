use pumpkin_data::fluid::Fluid;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::mob::Mob;
use crate::world::World;

const ATTEMPTS: usize = 10;

/// A random reachable land position around the mob.
pub fn land_random_pos(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
) -> Option<Vector3<f64>> {
    let restrict = mob_restricted(mob, f64::from(horizontal_dist));
    generate_random_pos(
        || {
            let direction = generate_random_direction(horizontal_dist, vertical_dist);
            let pos = generate_random_pos_toward_direction(
                mob,
                f64::from(horizontal_dist),
                restrict,
                direction,
            )?;
            move_pos_up_out_of_solid(mob, pos)
        },
        |_| 0.0,
    )
}

fn generate_random_pos(
    mut supplier: impl FnMut() -> Option<BlockPos>,
    position_weight: impl Fn(&BlockPos) -> f64,
) -> Option<Vector3<f64>> {
    let mut best_weight = f64::NEG_INFINITY;
    let mut best_pos = None;
    for _ in 0..ATTEMPTS {
        let Some(pos) = supplier() else {
            continue;
        };
        let weight = position_weight(&pos);
        if weight > best_weight {
            best_weight = weight;
            best_pos = Some(pos);
        }
    }
    best_pos.map(|pos| {
        Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y),
            f64::from(pos.0.z) + 0.5,
        )
    })
}

fn generate_random_direction(horizontal_dist: i32, vertical_dist: i32) -> BlockPos {
    BlockPos::new(
        rand::random_range(0..2 * horizontal_dist + 1) - horizontal_dist,
        rand::random_range(0..2 * vertical_dist + 1) - vertical_dist,
        rand::random_range(0..2 * horizontal_dist + 1) - horizontal_dist,
    )
}

fn mob_restricted(mob: &dyn Mob, horizontal_dist: f64) -> bool {
    let mob_entity = mob.get_mob_entity();
    let range = mob_entity
        .position_target_range
        .load(std::sync::atomic::Ordering::Relaxed);
    if range == -1 {
        return false;
    }
    let home = mob_entity.position_target.load();
    let home_center = Vector3::new(
        f64::from(home.0.x) + 0.5,
        f64::from(home.0.y) + 0.5,
        f64::from(home.0.z) + 0.5,
    );
    let max = f64::from(range) + horizontal_dist + 1.0;
    home_center.squared_distance_to_vec(&mob.get_entity().pos.load()) < max * max
}

fn generate_random_pos_toward_direction(
    mob: &dyn Mob,
    horizontal_dist: f64,
    restrict: bool,
    direction: BlockPos,
) -> Option<BlockPos> {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;
    let mob_pos = entity.pos.load();
    let world = entity.world.load();

    let mut xt = f64::from(direction.0.x);
    let mut zt = f64::from(direction.0.z);
    let has_home = mob_entity
        .position_target_range
        .load(std::sync::atomic::Ordering::Relaxed)
        != -1;
    if has_home && horizontal_dist > 1.0 {
        let home = mob_entity.position_target.load();
        if mob_pos.x > f64::from(home.0.x) {
            xt -= rand::random::<f64>() * horizontal_dist / 2.0;
        } else {
            xt += rand::random::<f64>() * horizontal_dist / 2.0;
        }
        if mob_pos.z > f64::from(home.0.z) {
            zt -= rand::random::<f64>() * horizontal_dist / 2.0;
        } else {
            zt += rand::random::<f64>() * horizontal_dist / 2.0;
        }
    }
    let pos = BlockPos::floored(
        xt + mob_pos.x,
        f64::from(direction.0.y) + mob_pos.y,
        zt + mob_pos.z,
    );

    let outside_limits = is_outside_build_height(&world, pos.0.y);
    let restricted = restrict && !mob_entity.is_in_position_target_range_pos(&pos);
    let unstable = !world.get_block_state(&pos.down()).is_full_cube();
    (!outside_limits && !restricted && !unstable).then_some(pos)
}

fn move_pos_up_out_of_solid(mob: &dyn Mob, pos: BlockPos) -> Option<BlockPos> {
    let entity = &mob.get_mob_entity().living_entity.entity;
    let world = entity.world.load();
    let max_y = world.dimension.min_y + world.dimension.height - 1;

    let mut pos = pos;
    if world.get_block_state(&pos).is_solid() {
        pos = pos.up();
        while pos.0.y <= max_y && world.get_block_state(&pos).is_solid() {
            pos = pos.up();
        }
    }

    let fluid = world.get_fluid(&pos);
    let is_water = fluid.id == Fluid::WATER.id || fluid.id == Fluid::FLOWING_WATER.id;
    (!is_water && !has_malus(mob, &pos)).then_some(pos)
}

fn has_malus(mob: &dyn Mob, pos: &BlockPos) -> bool {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;
    let mut context = PathfindingContext::new(entity.block_pos.load().0, entity.world.load_full());
    let path_type = context.get_land_node_type(pos.0);
    mob_entity
        .navigator
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_pathfinding_malus(path_type)
        != 0.0
}

const fn is_outside_build_height(world: &World, y: i32) -> bool {
    y < world.dimension.min_y || y >= world.dimension.min_y + world.dimension.height
}
