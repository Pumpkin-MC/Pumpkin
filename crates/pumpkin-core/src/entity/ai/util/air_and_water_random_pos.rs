use pumpkin_util::math::vector3::Vector3;

use super::{goal_utils, random_pos};
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::mob::Mob;

/// Picks a spot in open air or water, at `flying_height` blocks off the mob's level.
#[must_use]
pub fn get_pos(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
    flying_height: i32,
    x_dir: f64,
    z_dir: f64,
    max_xz_radians_difference: f64,
) -> Option<Vector3<f64>> {
    let restrict = goal_utils::mob_restricted(mob, f64::from(horizontal_dist));
    let world = mob.get_entity().world.load_full();
    let max_y = world.dimension.min_y + world.dimension.height - 1;
    let block_pos = mob.get_entity().block_pos.load();
    let mut context = PathfindingContext::new(
        Vector3::new(block_pos.0.x, block_pos.0.y, block_pos.0.z),
        world.clone(),
    );
    let mut rng = mob.get_random();

    random_pos::generate_random_pos(
        || {
            let direction = random_pos::generate_random_direction_within_radians(
                &mut rng,
                0.0,
                f64::from(horizontal_dist),
                vertical_dist,
                flying_height,
                x_dir,
                z_dir,
                max_xz_radians_difference,
            )?;
            let pos = random_pos::generate_random_pos_toward_direction(
                mob,
                f64::from(horizontal_dist),
                &mut rng,
                direction,
            );
            if goal_utils::is_outside_limits(&pos, &world)
                || goal_utils::is_restricted(restrict, mob, &pos)
            {
                return None;
            }
            let pos = random_pos::move_up_out_of_solid(pos, max_y, |candidate| {
                goal_utils::is_solid(&world, candidate)
            });
            (!goal_utils::has_malus(mob, &mut context, &pos)).then_some(pos)
        },
        |pos| f64::from(mob.get_walk_target_value(pos)),
    )
}
