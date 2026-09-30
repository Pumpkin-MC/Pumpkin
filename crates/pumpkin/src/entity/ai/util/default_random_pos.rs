use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use super::{goal_utils, random_pos};
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::mob::Mob;
use crate::world::World;

#[must_use]
pub fn get_pos(mob: &dyn Mob, horizontal_dist: i32, vertical_dist: i32) -> Option<Vector3<f64>> {
    let restrict = goal_utils::mob_restricted(mob, f64::from(horizontal_dist));
    let world = mob.get_entity().world.load_full();
    let mut context = new_context(mob, &world);
    let mut rng = mob.get_random();

    random_pos::generate_random_pos(
        || {
            let direction =
                random_pos::generate_random_direction(&mut rng, horizontal_dist, vertical_dist);
            toward_direction(
                mob,
                &world,
                &mut context,
                f64::from(horizontal_dist),
                restrict,
                direction,
                &mut rng,
            )
        },
        |pos| f64::from(mob.get_walk_target_value(pos)),
    )
}

#[must_use]
pub fn get_pos_towards(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
    towards_pos: Vector3<f64>,
    max_xz_radians_from_dir: f64,
) -> Option<Vector3<f64>> {
    let dir = towards_pos - mob.get_entity().pos.load();
    generate_in_direction(
        mob,
        horizontal_dist,
        vertical_dist,
        dir,
        max_xz_radians_from_dir,
    )
}

#[must_use]
pub fn get_pos_away(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
    avoid_pos: Vector3<f64>,
) -> Option<Vector3<f64>> {
    let dir_away = mob.get_entity().pos.load() - avoid_pos;
    generate_in_direction(
        mob,
        horizontal_dist,
        vertical_dist,
        dir_away,
        std::f64::consts::FRAC_PI_2,
    )
}

fn generate_in_direction(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
    dir: Vector3<f64>,
    max_xz_radians_from_dir: f64,
) -> Option<Vector3<f64>> {
    let restrict = goal_utils::mob_restricted(mob, f64::from(horizontal_dist));
    let world = mob.get_entity().world.load_full();
    let mut context = new_context(mob, &world);
    let mut rng = mob.get_random();

    random_pos::generate_random_pos(
        || {
            let direction = random_pos::generate_random_direction_within_radians(
                &mut rng,
                0.0,
                f64::from(horizontal_dist),
                vertical_dist,
                0,
                dir.x,
                dir.z,
                max_xz_radians_from_dir,
            )?;
            toward_direction(
                mob,
                &world,
                &mut context,
                f64::from(horizontal_dist),
                restrict,
                direction,
                &mut rng,
            )
        },
        |pos| f64::from(mob.get_walk_target_value(pos)),
    )
}

fn new_context(mob: &dyn Mob, world: &std::sync::Arc<World>) -> PathfindingContext {
    let block_pos = mob.get_entity().block_pos.load();
    PathfindingContext::new(
        Vector3::new(block_pos.0.x, block_pos.0.y, block_pos.0.z),
        world.clone(),
    )
}

fn toward_direction(
    mob: &dyn Mob,
    world: &World,
    context: &mut PathfindingContext,
    horizontal_dist: f64,
    restrict: bool,
    direction: Vector3<i32>,
    rng: &mut impl rand::RngExt,
) -> Option<BlockPos> {
    let pos =
        random_pos::generate_random_pos_toward_direction(mob, horizontal_dist, rng, direction);
    (!goal_utils::is_outside_limits(&pos, world)
        && !goal_utils::is_restricted(restrict, mob, &pos)
        && !goal_utils::is_not_stable(world, &pos)
        && !goal_utils::has_malus(mob, context, &pos))
    .then_some(pos)
}
