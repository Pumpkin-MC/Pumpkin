use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{goal_utils, land_random_pos, random_pos};
use crate::entity::ai::pathfinder::pathfinding_context::PathfindingContext;
use crate::entity::mob::Mob;

/// Picks a spot to hover over, somewhere between `hover_min_height` and
/// `hover_max_height` above the ground.
#[expect(clippy::too_many_arguments, reason = "mirrors the upstream signature")]
#[must_use]
pub fn get_pos(
    mob: &dyn Mob,
    horizontal_dist: i32,
    vertical_dist: i32,
    x_dir: f64,
    z_dir: f64,
    max_xz_radians_difference: f64,
    hover_max_height: i32,
    hover_min_height: i32,
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
                0,
                x_dir,
                z_dir,
                max_xz_radians_difference,
            )?;
            let pos = land_random_pos::toward_direction(
                mob,
                &world,
                f64::from(horizontal_dist),
                restrict,
                direction,
                &mut rng,
            )?;
            let above_solid_amount =
                rng.random_range(0..hover_max_height - hover_min_height + 1) + hover_min_height;
            let pos =
                random_pos::move_up_to_above_solid(pos, above_solid_amount, max_y, |candidate| {
                    goal_utils::is_solid(&world, candidate)
                });
            (!goal_utils::is_water(&world, &pos) && !goal_utils::has_malus(mob, &mut context, &pos))
                .then_some(pos)
        },
        |pos| f64::from(mob.get_walk_target_value(pos)),
    )
}
