/// Returns the amount of experience required to reach the next level from a given level
pub fn get_exp_to_next_level(current_level: i32) -> i32 {
    match current_level {
        0..=15 => 2 * current_level + 7,
        16..=30 => 5 * current_level - 38,
        _ => 9 * current_level - 158,
    }
}

/// Returns the total amount of experience points required to reach a specific level
pub fn get_total_exp_to_level(level: i32) -> i32 {
    match level {
        0..=16 => level * level + 6 * level,
        17..=31 => ((2.5 * f64::from(level * level)) - (40.5 * f64::from(level)) + 360.0) as i32,
        _ => ((4.5 * f64::from(level * level)) - (162.5 * f64::from(level)) + 2220.0) as i32,
    }
}

/// Calculates level from total experience points
pub fn get_level_from_total_exp(total_exp: i32) -> i32 {
    match total_exp {
        0..=352 => ((total_exp as f64 + 9.0).sqrt() - 3.0) as i32,
        353..=1507 => (81.0 + (total_exp as f64 - 7839.0) / 40.0).sqrt() as i32,
        _ => (325.0 + (total_exp as f64 - 54215.0) / 72.0).sqrt() as i32,
    }
}

/// Calculate experience progress (0.0 to 1.0) for a given total experience amount
pub fn get_progress_from_total_exp(total_exp: i32) -> f32 {
    let level = get_level_from_total_exp(total_exp);
    let next_level_exp = get_total_exp_to_level(level + 1);
    let current_level_exp = get_total_exp_to_level(level);
    (total_exp - current_level_exp) as f32 / (next_level_exp - current_level_exp) as f32
}
