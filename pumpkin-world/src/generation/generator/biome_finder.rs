use pumpkin_data::{chunk::Biome, dimension::Dimension};
use pumpkin_util::math::position::BlockPos;
use rayon::prelude::*;

use crate::{
    biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier},
    generation::noise::router::multi_noise_sampler::{
        MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
    },
};

use super::{VanillaGenerator, WorldGenerator};

#[derive(Debug, Clone, Copy)]
pub struct FoundBiome {
    pub position: BlockPos,
    pub biome: &'static Biome,
}

/// Finds the first matching biome using vanilla's horizontal spiral and
/// vertical out-from-origin traversal.
#[must_use]
pub fn find_closest_biome(
    generator: &WorldGenerator,
    origin: BlockPos,
    radius: i32,
    horizontal_interval: i32,
    vertical_interval: i32,
    target_biomes: &[u8],
) -> Option<FoundBiome> {
    if horizontal_interval <= 0 || vertical_interval <= 0 || target_biomes.is_empty() {
        return None;
    }

    let mut targets = [false; 256];
    for &id in target_biomes {
        targets[usize::from(id)] = true;
    }

    let dimension = generator.dimension();
    let vertical_samples = out_from_origin(
        origin.0.y,
        dimension.min_y + 1,
        dimension.min_y + dimension.height,
        vertical_interval,
    );

    match generator {
        WorldGenerator::Noise(generator) => find_noise_biome(
            generator,
            origin,
            radius / horizontal_interval,
            horizontal_interval,
            &vertical_samples,
            &targets,
        ),
        WorldGenerator::Flat(generator) => {
            let name = generator
                .biome
                .strip_prefix("minecraft:")
                .unwrap_or(&generator.biome);
            let biome = Biome::from_name(name).unwrap_or(&Biome::PLAINS);
            targets[usize::from(biome.id)].then(|| FoundBiome {
                position: BlockPos::new(origin.0.x, vertical_samples[0], origin.0.z),
                biome,
            })
        }
    }
}

/// Parallel version of [`find_closest_biome`] for use on the generation pool.
///
/// Columns are evaluated in bounded batches and `find_first` preserves the
/// exact vanilla traversal result even when later columns finish first.
#[must_use]
pub fn find_closest_biome_parallel(
    generator: &WorldGenerator,
    origin: BlockPos,
    radius: i32,
    horizontal_interval: i32,
    vertical_interval: i32,
    target_biomes: &[u8],
) -> Option<FoundBiome> {
    if horizontal_interval <= 0 || vertical_interval <= 0 || target_biomes.is_empty() {
        return None;
    }

    let mut targets = [false; 256];
    for &id in target_biomes {
        targets[usize::from(id)] = true;
    }

    let dimension = generator.dimension();
    let vertical_samples = out_from_origin(
        origin.0.y,
        dimension.min_y + 1,
        dimension.min_y + dimension.height,
        vertical_interval,
    );

    match generator {
        WorldGenerator::Noise(generator) => find_noise_biome_parallel(
            generator,
            origin,
            radius / horizontal_interval,
            horizontal_interval,
            &vertical_samples,
            &targets,
        ),
        WorldGenerator::Flat(generator) => {
            let name = generator
                .biome
                .strip_prefix("minecraft:")
                .unwrap_or(&generator.biome);
            let biome = Biome::from_name(name).unwrap_or(&Biome::PLAINS);
            targets[usize::from(biome.id)].then(|| FoundBiome {
                position: BlockPos::new(origin.0.x, vertical_samples[0], origin.0.z),
                biome,
            })
        }
    }
}

fn find_noise_biome(
    generator: &VanillaGenerator,
    origin: BlockPos,
    horizontal_radius: i32,
    horizontal_interval: i32,
    vertical_samples: &[i32],
    targets: &[bool; 256],
) -> Option<FoundBiome> {
    let mut sampler = MultiNoiseSampler::generate(
        &generator.base_router.multi_noise,
        &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
    );

    if generator.dimension == Dimension::THE_END {
        find_with_supplier(
            &TheEndBiomeSupplier,
            &mut sampler,
            origin,
            horizontal_radius,
            horizontal_interval,
            vertical_samples,
            targets,
        )
    } else if generator.dimension == Dimension::THE_NETHER {
        find_with_supplier(
            &MultiNoiseBiomeSupplier::NETHER,
            &mut sampler,
            origin,
            horizontal_radius,
            horizontal_interval,
            vertical_samples,
            targets,
        )
    } else {
        find_with_supplier(
            &MultiNoiseBiomeSupplier::OVERWORLD,
            &mut sampler,
            origin,
            horizontal_radius,
            horizontal_interval,
            vertical_samples,
            targets,
        )
    }
}

fn find_noise_biome_parallel(
    generator: &VanillaGenerator,
    origin: BlockPos,
    horizontal_radius: i32,
    horizontal_interval: i32,
    vertical_samples: &[i32],
    targets: &[bool; 256],
) -> Option<FoundBiome> {
    if generator.dimension == Dimension::THE_END {
        find_with_supplier_parallel(
            generator,
            &TheEndBiomeSupplier,
            origin,
            horizontal_radius,
            horizontal_interval,
            vertical_samples,
            targets,
        )
    } else if generator.dimension == Dimension::THE_NETHER {
        find_with_supplier_parallel(
            generator,
            &MultiNoiseBiomeSupplier::NETHER,
            origin,
            horizontal_radius,
            horizontal_interval,
            vertical_samples,
            targets,
        )
    } else {
        find_with_supplier_parallel(
            generator,
            &MultiNoiseBiomeSupplier::OVERWORLD,
            origin,
            horizontal_radius,
            horizontal_interval,
            vertical_samples,
            targets,
        )
    }
}

fn find_with_supplier(
    supplier: &dyn BiomeSupplier,
    sampler: &mut MultiNoiseSampler<'_>,
    origin: BlockPos,
    horizontal_radius: i32,
    horizontal_interval: i32,
    vertical_samples: &[i32],
    targets: &[bool; 256],
) -> Option<FoundBiome> {
    visit_spiral(horizontal_radius, |offset_x, offset_z| {
        let block_x = origin.0.x + offset_x * horizontal_interval;
        let block_z = origin.0.z + offset_z * horizontal_interval;
        let biome_x = block_x >> 2;
        let biome_z = block_z >> 2;

        for &block_y in vertical_samples {
            let biome = supplier.biome(biome_x, block_y >> 2, biome_z, sampler);
            if targets[usize::from(biome.id)] {
                return Some(FoundBiome {
                    position: BlockPos::new(block_x, block_y, block_z),
                    biome,
                });
            }
        }
        None
    })
}

fn find_with_supplier_parallel(
    generator: &VanillaGenerator,
    supplier: &(impl BiomeSupplier + Sync),
    origin: BlockPos,
    horizontal_radius: i32,
    horizontal_interval: i32,
    vertical_samples: &[i32],
    targets: &[bool; 256],
) -> Option<FoundBiome> {
    const COLUMN_BATCH_SIZE: usize = 256;

    let mut columns = Vec::with_capacity(
        usize::try_from((i64::from(horizontal_radius) * 2 + 1).pow(2)).unwrap_or_default(),
    );
    visit_spiral(horizontal_radius, |offset_x, offset_z| {
        columns.push((offset_x, offset_z));
        None::<()>
    });

    for batch in columns.chunks(COLUMN_BATCH_SIZE) {
        let found = batch
            .par_iter()
            .map_init(
                || {
                    MultiNoiseSampler::generate(
                        &generator.base_router.multi_noise,
                        &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
                    )
                },
                |sampler, &(offset_x, offset_z)| {
                    find_in_column(
                        supplier,
                        sampler,
                        origin,
                        offset_x,
                        offset_z,
                        horizontal_interval,
                        vertical_samples,
                        targets,
                    )
                },
            )
            .find_first(Option::is_some)
            .flatten();

        if found.is_some() {
            return found;
        }
    }

    None
}

#[expect(clippy::too_many_arguments)]
fn find_in_column(
    supplier: &impl BiomeSupplier,
    sampler: &mut MultiNoiseSampler<'_>,
    origin: BlockPos,
    offset_x: i32,
    offset_z: i32,
    horizontal_interval: i32,
    vertical_samples: &[i32],
    targets: &[bool; 256],
) -> Option<FoundBiome> {
    let block_x = origin.0.x + offset_x * horizontal_interval;
    let block_z = origin.0.z + offset_z * horizontal_interval;
    let biome_x = block_x >> 2;
    let biome_z = block_z >> 2;

    for &block_y in vertical_samples {
        let biome = supplier.biome(biome_x, block_y >> 2, biome_z, sampler);
        if targets[usize::from(biome.id)] {
            return Some(FoundBiome {
                position: BlockPos::new(block_x, block_y, block_z),
                biome,
            });
        }
    }

    None
}

fn visit_spiral<T>(radius: i32, mut visitor: impl FnMut(i32, i32) -> Option<T>) -> Option<T> {
    if let Some(result) = visitor(0, 0) {
        return Some(result);
    }

    let total = i64::from(radius * 2 + 1).pow(2);
    let directions = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut visited = 1i64;
    let mut x = 0;
    let mut z = 0;
    let mut direction = 0usize;
    let mut leg_length = 1;

    while visited < total {
        for _ in 0..leg_length {
            let (dx, dz) = directions[direction];
            x += dx;
            z += dz;
            visited += 1;
            if let Some(result) = visitor(x, z) {
                return Some(result);
            }
            if visited == total {
                return None;
            }
        }

        direction = (direction + 1) & 3;
        if direction & 1 == 0 {
            leg_length += 1;
        }
    }

    None
}

fn out_from_origin(origin: i32, lower: i32, upper: i32, step: i32) -> Vec<i32> {
    let origin = origin.clamp(lower, upper);
    let mut values = Vec::with_capacity(((upper - lower) / step + 2) as usize);
    values.push(origin);

    let mut distance = step;
    loop {
        let above = origin.saturating_add(distance);
        let below = origin.saturating_sub(distance);
        let has_above = above <= upper;
        let has_below = below >= lower;

        if has_above {
            values.push(above);
        }
        if has_below {
            values.push(below);
        }
        if !has_above && !has_below {
            break;
        }
        distance = distance.saturating_add(step);
    }

    values
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{chunk::Biome, dimension::Dimension};
    use pumpkin_util::{math::position::BlockPos, world_seed::Seed};

    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};

    use super::{
        FoundBiome, find_closest_biome, find_closest_biome_parallel, out_from_origin, visit_spiral,
    };

    #[test]
    fn spiral_matches_vanilla_order() {
        let mut positions = Vec::new();
        visit_spiral(1, |x, z| {
            positions.push((x, z));
            None::<()>
        });
        assert_eq!(
            positions,
            [
                (0, 0),
                (1, 0),
                (1, 1),
                (0, 1),
                (-1, 1),
                (-1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ]
        );
    }

    #[test]
    fn vertical_samples_move_out_from_origin() {
        assert_eq!(
            out_from_origin(64, -63, 320, 64),
            [64, 128, 0, 192, 256, 320]
        );
    }

    #[test]
    fn parallel_search_preserves_serial_result() {
        let generator = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(12_345),
            Dimension::OVERWORLD,
        )));
        let origin = BlockPos::new(173, 73, -241);

        for targets in [
            &[Biome::DESERT.id][..],
            &[Biome::PLAINS.id][..],
            &[Biome::BADLANDS.id, Biome::SWAMP.id][..],
        ] {
            let serial = find_closest_biome(&generator, origin, 512, 32, 64, targets);
            let parallel = find_closest_biome_parallel(&generator, origin, 512, 32, 64, targets);
            let comparable = |found: Option<FoundBiome>| {
                found.map(|found| {
                    (
                        found.position.0.x,
                        found.position.0.y,
                        found.position.0.z,
                        found.biome.id,
                    )
                })
            };

            assert_eq!(comparable(serial), comparable(parallel));
        }
    }
}
