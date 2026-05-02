use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use serde::Deserialize;

use super::int_provider::IntProvider;

#[derive(Deserialize, Clone, Debug)]
pub struct Experience {
    /// The experience points, represented as an `IntProvider`.
    pub experience: IntProvider,
}

impl ToTokens for Experience {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let experience = self.experience.to_token_stream();

        tokens.extend(quote! {
            Experience { experience: #experience }
        });
    }
}

/// Returns the number of points required to progress within a specific level.
///
/// # Arguments
/// * `level` – The level to calculate points for.
#[must_use]
pub const fn points_in_level(level: i32) -> i32 {
    match level {
        0..=15 => 2 * level + 7,
        16..=30 => 5 * level - 38,
        _ => 9 * level - 158,
    }
}

/// Calculates the total points required to reach a given level.
///
/// # Arguments
/// * `level` – The target level.
#[must_use]
pub fn points_to_level(level: i32) -> i32 {
    match level {
        ..=-1 => 0,
        0..=16 => level
            .saturating_mul(level)
            .saturating_add(level.saturating_mul(6)),
        17..=31 => {
            (2.5f64.mul_add(
                f64::from(level) * f64::from(level),
                -(40.5 * f64::from(level)),
            ) + 360.0) as i32
        }
        _ => {
            (4.5f64.mul_add(
                f64::from(level) * f64::from(level),
                -(162.5 * f64::from(level)),
            ) + 2220.0) as i32
        }
    }
}

/// Converts total experience points into a level and points within that level.
///
/// # Arguments
/// * `total_points` – The total accumulated experience points.
///
/// # Returns
/// A tuple `(level, points_into_level)` representing the current level and
/// remaining points within that level.
#[must_use]
pub fn total_to_level_and_points(total_points: i32) -> (i32, i32) {
    let level = match total_points {
        0..=352 => ((f64::from(total_points) + 9.0).sqrt() - 3.0) as i32,
        353..=1507 => (8.1 + (0.4 * (f64::from(total_points) - (7839.0 / 40.0))).sqrt()) as i32,
        _ => {
            ((325.0 / 18.0) + (2.0 / 9.0 * (f64::from(total_points) - (54215.0 / 72.0))).sqrt())
                as i32
        }
    };
    let level_start = points_to_level(level);
    let points_into_level = total_points.saturating_sub(level_start);

    (level, points_into_level)
}

/// Calculates the progress within a level as a value between 0.0 and 1.0.
///
/// # Arguments
/// * `points` – The points accumulated in the current level.
/// * `level` – The current level.
#[must_use]
pub fn progress_in_level(points: i32, level: i32) -> f32 {
    let max_points = points_in_level(level);
    let progress = (points as f32) / (max_points as f32);

    progress.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_to_level_handles_extremes_without_panic() {
        let _ = points_to_level(i32::MAX);
        let _ = points_to_level(i32::MIN);
        assert_eq!(points_to_level(-1), 0);
        assert_eq!(points_to_level(0), 0);
        assert_eq!(points_to_level(16), 16 * 16 + 6 * 16);
        // Vanilla: cumulative XP at level 30 is 1395, level 31 is 1507.
        assert_eq!(points_to_level(30), 1395);
        assert_eq!(points_to_level(31), 1507);
    }

    #[test]
    fn total_to_level_and_points_extremes() {
        assert_eq!(total_to_level_and_points(0), (0, 0));
        let (lvl_max, pts_max) = total_to_level_and_points(i32::MAX);
        assert!(lvl_max >= 0 && pts_max >= 0);
        let (lvl_neg, _) = total_to_level_and_points(-1);
        assert_eq!(lvl_neg, 0);
    }

    #[test]
    fn progress_in_level_is_clamped() {
        let p = progress_in_level(i32::MAX, 5);
        assert!((0.0..=1.0).contains(&p));
        let z = progress_in_level(0, 0);
        assert!((0.0..=1.0).contains(&z));
    }

    #[test]
    fn round_trip_total_to_level_and_back() {
        for total in [0i32, 100, 1000, 10_000, 100_000, 1_000_000] {
            let (lvl, pts) = total_to_level_and_points(total);
            let recomputed = points_to_level(lvl) + pts;
            assert_eq!(recomputed, total, "round-trip failed for total={total}");
        }
    }
}
