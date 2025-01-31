/// Returns the amount of experience required to reach the next level from a given level
#[must_use]
pub fn get_exp_to_next_level(current_level: i32) -> i32 {
    match current_level {
        0..=15 => 2 * current_level + 7,
        16..=30 => 5 * current_level - 38,
        _ => 9 * current_level - 158,
    }
}

/// Calculate total experience points from level and progress
#[must_use]
pub fn calculate_total_exp(level: i32, progress: f32) -> i32 {
    let level_base = match level {
        0..=16 => level * level + 6 * level,
        17..=31 => ((2.5 * f64::from(level * level)) - (40.5 * f64::from(level)) + 360.0) as i32,
        _ => ((4.5 * f64::from(level * level)) - (162.5 * f64::from(level)) + 2220.0) as i32,
    };

    let next_level_exp = get_exp_to_next_level(level);
    #[allow(clippy::cast_precision_loss)]
    let progress_exp = (next_level_exp as f32 * progress) as i32;

    level_base + progress_exp
}

/// Calculate level and progress from total experience points
#[must_use]
pub fn calculate_level_and_progress(total_exp: i32) -> (i32, f32) {
    let level = match total_exp {
        0..=352 => ((total_exp as f64 + 9.0).sqrt() - 3.0) as i32,
        353..=1507 => (81.0 + (total_exp as f64 - 7839.0) / 40.0).sqrt() as i32,
        _ => (325.0 + (total_exp as f64 - 54215.0) / 72.0).sqrt() as i32,
    };

    let level_start = match level {
        0..=16 => level * level + 6 * level,
        17..=31 => ((2.5 * f64::from(level * level)) - (40.5 * f64::from(level)) + 360.0) as i32,
        _ => ((4.5 * f64::from(level * level)) - (162.5 * f64::from(level)) + 2220.0) as i32,
    };

    let next_level_exp = get_exp_to_next_level(level);
    let exp_into_level = total_exp - level_start;
    
    #[allow(clippy::cast_precision_loss)]
    let progress = (exp_into_level as f32) / (next_level_exp as f32);

    (level, progress.clamp(0.0, 1.0))
}
