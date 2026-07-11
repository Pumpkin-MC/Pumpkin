use pumpkin_data::chunk::ParameterRange;
use pumpkin_util::math::vector2::Vector2;

/// Vanilla's world-spawn climate target (`OverworldBiomeBuilder#spawnTarget()`).
///
/// Expressed as `[temperature, humidity, continentalness, erosion, depth,
/// weirdness, offset]` ranges on the same x10000 fixed-point scale used
/// elsewhere in the biome parameter tree (see [`crate::biome::multi_noise::to_long`]).
///
/// Vanilla accepts weirdness in either a wide-negative or wide-positive band
/// (never near zero), so this is two alternative target points; the search
/// keeps whichever is closer for a given sample. Continentalness is
/// restricted to `-1100..=10000` (roughly "coast or further inland"), which
/// is what keeps the search out of oceans. Depth is pinned to the surface.
///
/// Values verified against the independently-maintained, vanilla-accurate
/// `Cubitect/cubiomes` reimplementation (`spawn_np` in `finders.c`), since
/// they aren't derived from any registry/datapack asset we generate from.
pub const OVERWORLD_SPAWN_TARGET: [[ParameterRange; 7]; 2] = [
    [
        ParameterRange::new(-10_000, 10_000), // temperature
        ParameterRange::new(-10_000, 10_000), // humidity
        ParameterRange::new(-1_100, 10_000),  // continentalness
        ParameterRange::new(-10_000, 10_000), // erosion
        ParameterRange::new(0, 0),            // depth
        ParameterRange::new(-10_000, -1_600), // weirdness, band A
        ParameterRange::new(0, 0),            // offset
    ],
    [
        ParameterRange::new(-10_000, 10_000), // temperature
        ParameterRange::new(-10_000, 10_000), // humidity
        ParameterRange::new(-1_100, 10_000),  // continentalness
        ParameterRange::new(-10_000, 10_000), // erosion
        ParameterRange::new(0, 0),            // depth
        ParameterRange::new(1_600, 10_000),   // weirdness, band B
        ParameterRange::new(0, 0),            // offset
    ],
];

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
