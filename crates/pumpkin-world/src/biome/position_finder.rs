use std::cell::RefCell;

use pumpkin_data::chunk::ParameterRange;
use pumpkin_util::math::vector2::Vector2;

use super::multi_noise::to_long;
use crate::generation::biome_coords;
use crate::generation::generator::VanillaGenerator;
use crate::generation::noise::router::multi_noise_sampler::{
    MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
};

/// Climate targets the overworld spawn search aims for.
///
/// Two points that differ only in weirdness, both keeping continentalness inland,
/// which is what steers the spawn away from oceans.
// OverworldBiomeBuilder.spawnTarget
#[must_use]
pub fn overworld_spawn_target() -> [[ParameterRange; 7]; 2] {
    const FULL_RANGE: (f32, f32) = (-1.0, 1.0);
    // Climate.Parameter.span(inlandContinentalness, FULL_RANGE) keeps the min of the
    // first and the max of the second.
    const INLAND: (f32, f32) = (-0.11, 1.0);
    const POINT_ZERO: (f32, f32) = (0.0, 0.0);

    let range = |(min, max): (f32, f32)| ParameterRange::new(to_long(min), to_long(max));

    [
        [
            range(FULL_RANGE),    // temperature
            range(FULL_RANGE),    // humidity
            range(INLAND),        // continentalness
            range(FULL_RANGE),    // erosion
            range(POINT_ZERO),    // depth
            range((-1.0, -0.16)), // weirdness
            range(POINT_ZERO),    // offset
        ],
        [
            range(FULL_RANGE),
            range(FULL_RANGE),
            range(INLAND),
            range(FULL_RANGE),
            range(POINT_ZERO),
            range((0.16, 1.0)),
            range(POINT_ZERO),
        ],
    ]
}

/// Picks the block position whose climate best matches [`overworld_spawn_target`].
///
/// This is the first half of vanilla's initial spawn search: it settles on a region,
/// not on a standing position. The caller still has to look for solid ground around
/// the returned position.
// Climate.Sampler.findSpawnPosition
#[must_use]
pub fn find_climate_spawn_position(generator: &VanillaGenerator) -> Vector2<i32> {
    let options = MultiNoiseSamplerBuilderOptions::new(1, 1, 1);
    let sampler = RefCell::new(MultiNoiseSampler::generate(
        &generator.base_router.multi_noise,
        &options,
    ));

    let sample = |x: i32, z: i32| {
        let point = sampler.borrow_mut().sample(
            biome_coords::from_block(x),
            0,
            biome_coords::from_block(z),
        );

        [
            point.temperature,
            point.humidity,
            point.continentalness,
            point.erosion,
            // Vanilla zeroes depth for the spawn search, unlike biome lookups.
            0,
            point.weirdness,
            0,
        ]
    };

    FittestPositionFinder::find_best_spawn_position(&overworld_spawn_target(), &sample)
}

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
                // Vanilla sums the *squared* distances (Climate.ParameterPoint#fitness).
                let distance = noise_ranges[i].calc_distance(sampled_noise[i]);
                current_dist += distance * distance;
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
