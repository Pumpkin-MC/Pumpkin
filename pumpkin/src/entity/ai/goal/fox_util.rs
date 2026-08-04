use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::world::World;

/// `Fox.isPathClear`'s 6-sample ground-plane line from `from` to `to`, as (x, z) offsets from
/// `from`. Split out from [`fox_path_is_clear`] so the sampling geometry (in particular the
/// `slope == 0.0` special case, needed to avoid a `x / 0.0` when `to` is due north/south of
/// `from`) is testable without a live `World`.
#[must_use]
fn sample_offsets(from: Vector3<f64>, to: Vector3<f64>) -> [(f64, f64); 6] {
    let zdiff = to.z - from.z;
    let xdiff = to.x - from.x;
    let slope = zdiff / xdiff;

    std::array::from_fn(|i| {
        let z = if slope == 0.0 {
            0.0
        } else {
            zdiff * (i as f64 / 6.0)
        };
        let x = if slope == 0.0 {
            xdiff * (i as f64 / 6.0)
        } else {
            z / slope
        };
        (x, z)
    })
}

/// `Fox.isPathClear`: samples 6 points along the ground-plane line from `from` to `to`.
///
/// Checks 3 blocks of vertical clearance above `from`'s current height at each sample. Used by
/// both the stalk and pounce goals to make sure the fox has a clear line to its prey.
#[must_use]
pub fn fox_path_is_clear(world: &World, from: Vector3<f64>, to: Vector3<f64>) -> bool {
    for (x, z) in sample_offsets(from, to) {
        for j in 1..4 {
            let pos = BlockPos(Vector3::new(
                (from.x + x).floor() as i32,
                (from.y + f64::from(j)).floor() as i32,
                (from.z + z).floor() as i32,
            ));
            if !world.get_block_state(&pos).replaceable() {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zero_slope_samples_stay_on_the_x_axis() {
        // Target directly along +x, same z: vanilla special-cases `slope == 0.0` to avoid
        // `z / slope` dividing by zero, and instead walks `x` linearly.
        let from = Vector3::new(0.0, 64.0, 0.0);
        let to = Vector3::new(6.0, 64.0, 0.0);
        let offsets = sample_offsets(from, to);
        assert_eq!(offsets[0], (0.0, 0.0));
        assert_eq!(offsets[3], (3.0, 0.0));
        assert!(offsets.iter().all(|&(_, z)| z == 0.0));
    }

    #[test]
    fn diagonal_samples_interpolate_both_axes() {
        let from = Vector3::new(0.0, 64.0, 0.0);
        let to = Vector3::new(6.0, 64.0, 6.0);
        let offsets = sample_offsets(from, to);
        assert_eq!(offsets[0], (0.0, 0.0));
        for &(x, z) in &offsets {
            assert!((x - z).abs() < 1e-9);
        }
    }
}
