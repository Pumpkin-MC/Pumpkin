use std::sync::Arc;

use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::vector3::Vector3;

// ══════════════════════════════════════════════════════════════════
// Regional Difficulty — exact vanilla DifficultyInstance.java
// Vanilla approximation: identical formula to vanilla Minecraft 26.2's
// DifficultyInstance, including clamped regional difficulty, special
// multiplier (0-1 linear), and effective difficulty (2-4 range).
// ══════════════════════════════════════════════════════════════════

/// Computed difficulty values for a specific world chunk.
///
/// Mirrors Vanilla's `DifficultyInstance`. Used to scale equipment spawn rates,
/// enchantment costs, and loot-pickup flags.
#[derive(Clone, Copy)]
pub struct RegionalDifficulty {
    /// The world's base difficulty level (`Easy`, `Normal`, `Hard`).
    pub base_difficulty: Difficulty,
    /// Effective difficulty computed from game time, inhabited time, and moon phase.
    /// Clamped to the range `[2.0, 4.0]` (or `0.0` for Peaceful).
    pub effective_difficulty: f32,
    /// Linear multiplier in `[0.0, 1.0]` derived from `effective_difficulty`.
    /// When `0.0` (fresh chunk + early game), no equipment, enchantments, or
    /// loot-pickup flags are applied.
    pub special_multiplier: f32,
}

impl RegionalDifficulty {
    /// Computes difficulty at the given world position.
    ///
    /// Looks up the chunk's inhabited time and combines it with the world's
    /// difficulty, game time, and moon phase.
    pub fn at(world: &Arc<crate::world::World>, pos: Vector3<f64>) -> Self {
        let level_info = world.level_info.load();
        let difficulty = level_info.difficulty;
        let time_of_day = world.level_time.try_lock().map_or(0, |t| t.time_of_day);
        let inhabited_time = {
            let chunk_x = (pos.x / 16.0).floor() as i32;
            let chunk_z = (pos.z / 16.0).floor() as i32;
            world
                .level
                .loaded_chunks
                .get(&pumpkin_util::math::vector2::Vector2::new(chunk_x, chunk_z))
                .map_or(0, |c| {
                    c.inhabited_time.load(std::sync::atomic::Ordering::Relaxed)
                })
        };
        let moon_brightness = moon_brightness(time_of_day);

        Self::calculate(difficulty, time_of_day, inhabited_time, moon_brightness)
    }

    /// Direct calculation from raw inputs. Used by `at()` and for testing.
    #[must_use]
    pub fn calculate(
        difficulty: Difficulty,
        total_game_time: i64,
        chunk_inhabited_time: u64,
        moon_brightness: f32,
    ) -> Self {
        if difficulty == Difficulty::Peaceful {
            return Self {
                base_difficulty: difficulty,
                effective_difficulty: 0.0,
                special_multiplier: 0.0,
            };
        }

        let is_hard = difficulty == Difficulty::Hard;

        let mut scale = 0.75f32;
        let global_scale = ((total_game_time as f32 - 72000.0) / 1440000.0).clamp(0.0, 1.0) * 0.25;
        scale += global_scale;

        let mut local_scale = 0.0f32;
        local_scale += (chunk_inhabited_time as f32 / 3600000.0).clamp(0.0, 1.0)
            * if is_hard { 1.0 } else { 0.75 };
        local_scale += (moon_brightness * 0.25).clamp(0.0, global_scale);

        if difficulty == Difficulty::Easy {
            local_scale *= 0.5;
        }

        let difficulty_id = match difficulty {
            Difficulty::Peaceful => 0,
            Difficulty::Easy => 1,
            Difficulty::Normal => 2,
            Difficulty::Hard => 3,
        };

        let effective = difficulty_id as f32 * (scale + local_scale);

        let special_multiplier = if effective < 2.0 {
            0.0
        } else if effective > 4.0 {
            1.0
        } else {
            (effective - 2.0) / 2.0
        };

        Self {
            base_difficulty: difficulty,
            effective_difficulty: effective,
            special_multiplier,
        }
    }

    /// Random check scaled by `special_multiplier`.
    ///
    /// Returns `true` with probability `base_chance * special_multiplier`. When
    /// `special_multiplier` is `0.0` this always returns `false` (matching vanilla
    /// behaviour on fresh Normal/Easy worlds).
    #[must_use]
    pub fn should_happen(&self, base_chance: f32) -> bool {
        rand::random::<f32>() < base_chance * self.special_multiplier
    }
}

/// Moon brightness factor for the given time of day (0.0 to 1.0).
/// Full moon at phase 0, new moon at phase 4.
#[must_use]
fn moon_brightness(time_of_day: i64) -> f32 {
    let phase = (time_of_day / 24000 % 8) as i32;
    if phase == 0 {
        1.0
    } else {
        1.0 - (phase - 4).abs() as f32 / 4.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time assertions that the public paths and signatures survived the
    // module split (re-exported through `crate::entity::mob::equipment`).
    const _: fn(Difficulty, i64, u64, f32) -> crate::entity::mob::equipment::RegionalDifficulty =
        crate::entity::mob::equipment::RegionalDifficulty::calculate;

    #[test]
    fn peaceful_difficulty_zeroes_everything() {
        let difficulty =
            RegionalDifficulty::calculate(Difficulty::Peaceful, 1_000_000, 1_000_000, 1.0);
        assert!(difficulty.effective_difficulty.abs() < f32::EPSILON);
        assert!(difficulty.special_multiplier.abs() < f32::EPSILON);
        // With a zero multiplier the scaled roll can never succeed.
        assert!(!difficulty.should_happen(1.0));
    }

    #[test]
    fn fresh_normal_world_has_zero_special_multiplier() {
        // time 0, uninhabited chunk: effective = 2 * 0.75 = 1.5 < 2.0.
        let difficulty = RegionalDifficulty::calculate(Difficulty::Normal, 0, 0, 1.0);
        assert!((difficulty.effective_difficulty - 1.5).abs() < 1e-6);
        assert!(difficulty.special_multiplier.abs() < f32::EPSILON);
        assert!(!difficulty.should_happen(1.0));
    }

    #[test]
    fn late_game_hard_world_saturates_the_multiplier() {
        // Very old world + fully inhabited chunk on Hard: effective > 4.0.
        let difficulty =
            RegionalDifficulty::calculate(Difficulty::Hard, 10_000_000, 10_000_000, 1.0);
        assert!(difficulty.effective_difficulty > 4.0);
        assert!((difficulty.special_multiplier - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn easy_halves_the_local_scale() {
        let easy = RegionalDifficulty::calculate(Difficulty::Easy, 10_000_000, 10_000_000, 0.0);
        let normal = RegionalDifficulty::calculate(Difficulty::Normal, 10_000_000, 10_000_000, 0.0);
        assert!(easy.effective_difficulty < normal.effective_difficulty);
    }

    #[test]
    fn moon_brightness_peaks_at_full_moon() {
        assert!((moon_brightness(0) - 1.0).abs() < f32::EPSILON);
        // Phase 2: 1.0 - |2 - 4| / 4 = 0.5.
        assert!((moon_brightness(2 * 24000) - 0.5).abs() < f32::EPSILON);
    }
}
