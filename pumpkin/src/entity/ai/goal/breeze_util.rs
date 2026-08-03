//! Shared helpers ported from `BreezeUtil.java`, used by the jump and slide goals.

use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

fn direction_from_yaw(yaw_deg: f32) -> Vector3<f64> {
    let yaw_rad = f64::from(yaw_deg.to_radians());
    Vector3::new(-yaw_rad.sin(), 0.0, yaw_rad.cos())
}

/// `BreezeUtil.randomPointBehindTarget`.
///
/// A point behind the target's facing direction, spread by a gaussian sample (stddev
/// 45 degrees) around the 180-degree (directly behind) angle, at a distance uniformly
/// sampled in `[4, 8)`.
pub fn random_point_behind_target(
    target_pos: Vector3<f64>,
    target_head_yaw: f32,
    random: &mut RandomGenerator,
) -> Vector3<f64> {
    let view_angle = target_head_yaw + 180.0 + (random.next_gaussian() as f32) * 90.0 / 2.0;
    let radius = 4.0 + f64::from(random.next_f32()) * 4.0;
    target_pos.add(&direction_from_yaw(view_angle).multiply(radius, radius, radius))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::random::xoroshiro128::Xoroshiro;

    #[test]
    fn stays_within_the_vanilla_radius_band() {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(1234));
        for _ in 0..64 {
            let target = Vector3::new(0.0, 64.0, 0.0);
            let point = random_point_behind_target(target, 0.0, &mut random);
            let dist = point.sub(&target).length();
            // radius is sampled in [4, 8); allow slack for the gaussian spread
            // slightly changing the effective horizontal distance is not expected
            // here since the direction vector itself has length `radius`.
            assert!((4.0..8.0).contains(&dist), "distance {dist} out of range");
        }
    }
}
