use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::mob::Mob;

pub const RANDOM_POS_ATTEMPTS: usize = 10;
const SQRT_OF_TWO: f64 = std::f64::consts::SQRT_2;

pub fn generate_random_direction(
    rng: &mut impl RngExt,
    horizontal_dist: i32,
    vertical_dist: i32,
) -> Vector3<i32> {
    Vector3::new(
        rng.random_range(0..2 * horizontal_dist + 1) - horizontal_dist,
        rng.random_range(0..2 * vertical_dist + 1) - vertical_dist,
        rng.random_range(0..2 * horizontal_dist + 1) - horizontal_dist,
    )
}

#[allow(clippy::too_many_arguments, reason = "mirrors the upstream signature")]
pub fn generate_random_direction_within_radians(
    rng: &mut impl RngExt,
    min_horizontal_dist: f64,
    max_horizontal_dist: f64,
    vertical_dist: i32,
    flying_height: i32,
    x_dir: f64,
    z_dir: f64,
    max_xz_radians_from_dir: f64,
) -> Option<Vector3<i32>> {
    let y_radians_center = z_dir.atan2(x_dir) - std::f64::consts::FRAC_PI_2;
    let y_radians = f64::from(2.0f32.mul_add(rng.random::<f32>(), -1.0))
        .mul_add(max_xz_radians_from_dir, y_radians_center);
    let lerped = (max_horizontal_dist - min_horizontal_dist)
        .mul_add(rng.random::<f64>().sqrt(), min_horizontal_dist);
    let dist = lerped * SQRT_OF_TWO;

    let xt = -dist * y_radians.sin();
    let zt = dist * y_radians.cos();
    if xt.abs() > max_horizontal_dist || zt.abs() > max_horizontal_dist {
        return None;
    }

    let yt = rng.random_range(0..2 * vertical_dist + 1) - vertical_dist + flying_height;
    Some(Vector3::new(
        xt.floor() as i32,
        f64::from(yt).floor() as i32,
        zt.floor() as i32,
    ))
}

pub fn move_up_out_of_solid(
    pos: BlockPos,
    max_y: i32,
    solidity_tester: impl Fn(&BlockPos) -> bool,
) -> BlockPos {
    if !solidity_tester(&pos) {
        return pos;
    }

    let mut on_ground = pos.up();
    while on_ground.0.y <= max_y && solidity_tester(&on_ground) {
        on_ground = on_ground.up();
    }
    on_ground
}

/// Pushes `pos` up out of any solid stack, then up to `above_solid_amount` blocks
/// further while there is room.
pub fn move_up_to_above_solid(
    pos: BlockPos,
    above_solid_amount: i32,
    max_y: i32,
    solidity_tester: impl Fn(&BlockPos) -> bool,
) -> BlockPos {
    if !solidity_tester(&pos) {
        return pos;
    }

    let mut current = pos.up();
    while current.0.y <= max_y && solidity_tester(&current) {
        current = current.up();
    }

    let first_non_solid_y = current.0.y;
    while current.0.y <= max_y && current.0.y - first_non_solid_y < above_solid_amount {
        current = current.up();
        if solidity_tester(&current) {
            current = current.down();
            break;
        }
    }

    current
}

/// Ten attempts, keep the best weighted one.
pub fn generate_random_pos(
    mut pos_supplier: impl FnMut() -> Option<BlockPos>,
    position_weight: impl Fn(&BlockPos) -> f64,
) -> Option<Vector3<f64>> {
    let mut best_weight = f64::NEG_INFINITY;
    let mut best_pos = None;

    for _ in 0..RANDOM_POS_ATTEMPTS {
        if let Some(pos) = pos_supplier() {
            let weight = position_weight(&pos);
            if weight > best_weight {
                best_weight = weight;
                best_pos = Some(pos);
            }
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

/// Biases the offset away from the mob's home so restricted mobs stay in their area.
pub fn generate_random_pos_toward_direction(
    mob: &dyn Mob,
    xz_dist: f64,
    rng: &mut impl RngExt,
    direction: Vector3<i32>,
) -> BlockPos {
    let mob_entity = mob.get_mob_entity();
    let mob_pos = mob_entity.living_entity.entity.pos.load();
    let mut xt = f64::from(direction.x);
    let mut zt = f64::from(direction.z);

    if mob_entity.has_position_target() && xz_dist > 1.0 {
        let center = mob_entity.position_target.load();
        if mob_pos.x > f64::from(center.0.x) {
            xt -= rng.random::<f64>() * xz_dist / 2.0;
        } else {
            xt += rng.random::<f64>() * xz_dist / 2.0;
        }

        if mob_pos.z > f64::from(center.0.z) {
            zt -= rng.random::<f64>() * xz_dist / 2.0;
        } else {
            zt += rng.random::<f64>() * xz_dist / 2.0;
        }
    }

    BlockPos::new(
        (xt + mob_pos.x).floor() as i32,
        (f64::from(direction.y) + mob_pos.y).floor() as i32,
        (zt + mob_pos.z).floor() as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn generate_random_pos_keeps_the_best_weighted_candidate() {
        let attempt = Cell::new(0);
        let pos = generate_random_pos(
            || {
                let i = attempt.get();
                attempt.set(i + 1);
                Some(BlockPos::new(i, 0, 0))
            },
            // Best score sits in the middle, so neither first nor last wins by accident.
            |pos| -f64::from((pos.0.x - 4).abs()),
        );

        assert_eq!(attempt.get(), RANDOM_POS_ATTEMPTS as i32);
        // Bottom centre of the block.
        assert_eq!(pos, Some(Vector3::new(4.5, 0.0, 0.5)));
    }

    #[test]
    fn generate_random_pos_gives_up_when_every_attempt_fails() {
        assert_eq!(generate_random_pos(|| None, |_| 0.0), None);
    }

    #[test]
    fn move_up_out_of_solid_stops_at_the_first_free_block() {
        let solid = |pos: &BlockPos| pos.0.y < 68;

        assert_eq!(
            move_up_out_of_solid(BlockPos::new(0, 64, 0), 128, solid),
            BlockPos::new(0, 68, 0)
        );
        // Already free: the position is returned untouched.
        assert_eq!(
            move_up_out_of_solid(BlockPos::new(0, 70, 0), 128, solid),
            BlockPos::new(0, 70, 0)
        );
        // Solid all the way to the build limit: the search stops there.
        assert_eq!(
            move_up_out_of_solid(BlockPos::new(0, 64, 0), 66, solid),
            BlockPos::new(0, 67, 0)
        );
    }
}
