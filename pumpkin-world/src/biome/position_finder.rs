use pumpkin_data::chunk::ParameterRange;
use pumpkin_util::math::vector2::Vector2;

pub struct FittestPositionFinderResult {
    pub location: Vector2<i32>,
    pub fitness: i64,
}

pub struct FittestPositionFinder;

impl FittestPositionFinder {
    pub fn find_best_spawn_position(
        target_noises: &[[ParameterRange; 7]],
        sampler: &dyn Fn(i32, i32) -> [i64; 7],
    ) -> Vector2<i32> {
        let mut best_result = Self::calculate_fitness(target_noises, sampler, 0, 0);

        Self::find_fittest(target_noises, sampler, &mut best_result, 2048.0, 512.0);
        Self::find_fittest(target_noises, sampler, &mut best_result, 512.0, 32.0);

        best_result.location
    }

    fn find_fittest(
        noises: &[[ParameterRange; 7]],
        sampler: &dyn Fn(i32, i32) -> [i64; 7],
        best_result: &mut FittestPositionFinderResult,
        max_distance: f32,
        step: f32,
    ) {
        let mut angle = 0.0f32;
        let mut distance = step;
        let center = best_result.location;

        while distance <= max_distance {
            let x = center.x + (angle.sin() * distance) as i32;
            let z = center.y + (angle.cos() * distance) as i32;

            let result = Self::calculate_fitness(noises, sampler, x, z);
            if result.fitness < best_result.fitness {
                *best_result = result;
            }

            angle += step / distance;
            if angle > std::f32::consts::TAU {
                angle = 0.0;
                distance += step;
            }
        }
    }

    fn calculate_fitness(
        noises: &[[ParameterRange; 7]],
        sampler: &dyn Fn(i32, i32) -> [i64; 7],
        x: i32,
        z: i32,
    ) -> FittestPositionFinderResult {
        let sampled_noise = sampler(x, z);
        let mut min_squared_dist = i64::MAX;

        for noise_ranges in noises {
            let mut current_dist = 0i64;
            for i in 0..7 {
                current_dist += noise_ranges[i].calc_distance(sampled_noise[i]);
            }
            min_squared_dist = min_squared_dist.min(current_dist);
        }

        let origin_dist_sq = (x as i64 * x as i64) + (z as i64 * z as i64);
        let fitness = min_squared_dist * (2048 * 2048) + origin_dist_sq;

        FittestPositionFinderResult {
            location: Vector2::new(x, z),
            fitness,
        }
    }
}
