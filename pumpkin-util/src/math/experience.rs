#[cfg(feature = "codegen")]
use proc_macro2::TokenStream;
#[cfg(feature = "codegen")]
use quote::{ToTokens, quote};
use serde::Deserialize;

use super::int_provider::IntProvider;

#[derive(Deserialize, Clone, Debug)]
pub struct Experience {
    /// The experience points, represented as an `IntProvider`.
    pub experience: IntProvider,
}

#[cfg(feature = "codegen")]
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
        0..=16 => level * level + 6 * level,
        17..=31 => {
            (2.5f64.mul_add(f64::from(level * level), -(40.5 * f64::from(level))) + 360.0) as i32
        }
        _ => {
            (4.5f64.mul_add(f64::from(level * level), -(162.5 * f64::from(level))) + 2220.0) as i32
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
    let points_into_level = total_points - level_start;

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

/// The highest level for which `points_in_level` still fits in an `i32`
/// (`9 * level - 158 <= i32::MAX`).
pub const MAX_LEVEL: i32 = i32::MAX / 9;

/// Applies a point delta to a `(level, points_into_level)` pair, carrying
/// across level boundaries like vanilla's `giveExperiencePoints`.
///
/// Unlike converting through total accumulated points, this never saturates
/// once the total exceeds `i32::MAX`, so levels keep increasing.
///
/// # Returns
/// The new `(level, points_into_level)` pair, with
/// `0 <= points_into_level < points_in_level(level)`.
#[must_use]
pub fn add_points_to_level(level: i32, points: i32, added: i32) -> (i32, i32) {
    let mut level = level.max(0);
    let mut points = i64::from(points.max(0)) + i64::from(added);

    while points < 0 {
        if level == 0 {
            return (0, 0);
        }
        level -= 1;
        points += i64::from(points_in_level(level));
    }

    loop {
        let needed = i64::from(points_in_level(level));
        if points < needed {
            break;
        }
        if level >= MAX_LEVEL {
            // Saturate below the next level instead of overflowing the level math.
            points = needed - 1;
            break;
        }
        points -= needed;
        level += 1;
    }

    (level, points as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_points_within_level() {
        assert_eq!(add_points_to_level(0, 0, 5), (0, 5));
        assert_eq!(add_points_to_level(3, 2, 4), (3, 6));
    }

    #[test]
    fn add_points_carries_levels_up() {
        // Level 0 needs 7 points.
        assert_eq!(add_points_to_level(0, 0, 7), (1, 0));
        assert_eq!(add_points_to_level(0, 0, 8), (1, 1));
        // 7 + 9 = 16 points reaches exactly level 2.
        assert_eq!(add_points_to_level(0, 0, 16), (2, 0));
        assert_eq!(points_to_level(2), 16);
    }

    #[test]
    fn add_points_carries_levels_down() {
        // Removing one point from the start of level 1 lands at 6/7 of level 0.
        assert_eq!(add_points_to_level(1, 0, -1), (0, 6));
        // Removing more points than the player has clamps at zero.
        assert_eq!(add_points_to_level(0, 3, -10), (0, 0));
        assert_eq!(add_points_to_level(2, 0, -1000), (0, 0));
    }

    #[test]
    fn add_points_keeps_totals_consistent() {
        let (level, points) = add_points_to_level(0, 0, i32::MAX);
        assert_eq!(
            i64::from(points_to_level(level)) + i64::from(points),
            i64::from(i32::MAX)
        );
        assert!(points >= 0 && points < points_in_level(level));
    }

    #[test]
    fn add_points_exceeds_i32_total() {
        // Issue #2094: levels must keep increasing after the accumulated
        // total passes `i32::MAX`.
        let (level_a, points_a) = add_points_to_level(0, 0, i32::MAX);
        let (level_b, points_b) = add_points_to_level(level_a, points_a, i32::MAX);
        assert!(level_b > level_a);
        assert!(points_b >= 0 && points_b < points_in_level(level_b));
    }
}
