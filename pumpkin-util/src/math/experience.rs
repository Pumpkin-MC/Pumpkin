/// Get the number of points in a level
pub fn points_in_level(level: i32) -> i32 {
    match level {
        0..=15 => 2 * level + 7,
        16..=30 => 5 * level - 38,
        _ => 9 * level - 158,
    }
}

/// Calculate the total number of points to reach a level
pub fn points_to_level(level: i32) -> i32 {
    match level {
        0..=15 => (level * level + 6 * level) / 2,
        16..=30 => ((2.5 * f64::from(level * level)) - (40.5 * f64::from(level)) + 360.0) as i32,
        _ => ((4.5 * f64::from(level * level)) - (162.5 * f64::from(level)) + 2220.0) as i32,
    }
}

/// Calculate level and points from total points
pub fn total_to_level_and_points(total_points: i32) -> (i32, i32) {
    let level = match total_points {
        0..=352 => ((total_points as f64 + 9.0).sqrt() - 3.0) as i32,
        353..=1507 => (81.0 + (total_points as f64 - 7839.0) / 40.0).sqrt() as i32,
        _ => (325.0 + (total_points as f64 - 54215.0) / 72.0).sqrt() as i32,
    };

    let level_start = points_to_level(level);
    let points_into_level = total_points - level_start;

    (level, points_into_level)
}

/// Calculate progress (0.0 to 1.0) from points within a level
pub fn progress_in_level(points: i32, level: i32) -> f32 {
    let max_points = points_in_level(level);

    let progress = (points as f32) / (max_points as f32);
    progress.clamp(0.0, 1.0)
}
