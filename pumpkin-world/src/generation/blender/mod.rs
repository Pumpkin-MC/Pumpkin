pub mod blending_data;

use std::f64;

use crate::biome::BiomeSupplier;
use crate::generation::biome_coords;
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
use crate::generation::proto_chunk::GenerationCache;
use blending_data::BlendingData;
use pumpkin_data::chunk::Biome;
use pumpkin_data::noise_parameter::DoublePerlinNoiseParameters;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use rustc_hash::FxHashMap;

pub struct BlendingOutput {
    pub alpha: f64,
    pub blending_offset: f64,
}

pub struct Blender {
    height_and_biome_blending_data: FxHashMap<u64, BlendingData>,
    density_blending_data: FxHashMap<u64, BlendingData>,
}

const HEIGHT_BLENDING_RANGE_CELLS: i32 = (7 << 2) - 1; // QuartPos.fromSection(7) - 1
const HEIGHT_BLENDING_RANGE_CHUNKS: i32 = (HEIGHT_BLENDING_RANGE_CELLS + 3) >> 2; // QuartPos.toSection(...)
const DENSITY_BLENDING_RANGE_CELLS: i32 = 2;
const DENSITY_BLENDING_RANGE_CHUNKS: i32 = 5 >> 2; // QuartPos.toSection(5)

impl Blender {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            height_and_biome_blending_data: FxHashMap::default(),
            density_blending_data: FxHashMap::default(),
        }
    }

    pub fn of<C: GenerationCache>(cache: &C) -> Self {
        let center_chunk = cache.get_center_chunk();
        let center_x = center_chunk.x;
        let center_z = center_chunk.z;

        let mut height_and_biome_data = FxHashMap::default();
        let mut density_data = FxHashMap::default();

        let max_dist_sq = (HEIGHT_BLENDING_RANGE_CHUNKS + 1) * (HEIGHT_BLENDING_RANGE_CHUNKS + 1);

        for dx in -HEIGHT_BLENDING_RANGE_CHUNKS..=HEIGHT_BLENDING_RANGE_CHUNKS {
            for dz in -HEIGHT_BLENDING_RANGE_CHUNKS..=HEIGHT_BLENDING_RANGE_CHUNKS {
                if dx * dx + dz * dz <= max_dist_sq {
                    let chunk_x = center_x + dx;
                    let chunk_z = center_z + dz;

                    if let Some(blending_data) = cache.get_blending_data(chunk_x, chunk_z) {
                        let packed = (chunk_x as u32 as u64) | ((chunk_z as u32 as u64) << 32);
                        height_and_biome_data.insert(packed, blending_data.clone());

                        if (-DENSITY_BLENDING_RANGE_CHUNKS..=DENSITY_BLENDING_RANGE_CHUNKS)
                            .contains(&dx)
                            && (-DENSITY_BLENDING_RANGE_CHUNKS..=DENSITY_BLENDING_RANGE_CHUNKS)
                                .contains(&dz)
                        {
                            density_data.insert(packed, blending_data.clone());
                        }
                    }
                }
            }
        }

        if height_and_biome_data.is_empty() && density_data.is_empty() {
            Self::empty()
        } else {
            Self {
                height_and_biome_blending_data: height_and_biome_data,
                density_blending_data: density_data,
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.height_and_biome_blending_data.is_empty() && self.density_blending_data.is_empty()
    }

    #[must_use]
    pub fn blend_offset_and_factor(&self, block_x: i32, block_z: i32) -> BlendingOutput {
        let cell_x = biome_coords::from_block(block_x);
        let cell_z = biome_coords::from_block(block_z);

        let fixed_height = self.get_blending_data_value(
            cell_x,
            0,
            cell_z,
            blending_data::BlendingData::get_height,
        );

        if fixed_height != f64::MAX {
            return BlendingOutput {
                alpha: 0.0,
                blending_offset: Self::height_to_offset(fixed_height),
            };
        }

        let mut total_weight = 0.0;
        let mut weighted_heights = 0.0;
        let mut closest_distance = f64::INFINITY;

        for (&packed_pos, blending_data) in &self.height_and_biome_blending_data {
            let chunk_x = (packed_pos & 0xFFFFFFFF) as i32;
            let chunk_z = (packed_pos >> 32) as i32;

            blending_data.iterate_heights(
                biome_coords::from_chunk(chunk_x),
                biome_coords::from_chunk(chunk_z),
                |test_cell_x, test_cell_z, height| {
                    let dx = (cell_x - test_cell_x) as f64;
                    let dz = (cell_z - test_cell_z) as f64;
                    let distance = dx.hypot(dz);

                    if distance <= HEIGHT_BLENDING_RANGE_CELLS as f64 {
                        if distance < closest_distance {
                            closest_distance = distance;
                        }

                        let weight = 1.0 / (distance * distance * distance * distance);
                        weighted_heights += height * weight;
                        total_weight += weight;
                    }
                },
            );
        }

        if closest_distance == f64::INFINITY {
            BlendingOutput {
                alpha: 1.0,
                blending_offset: 0.0,
            }
        } else {
            let average_height = weighted_heights / total_weight;
            let mut alpha =
                (closest_distance / (HEIGHT_BLENDING_RANGE_CELLS + 1) as f64).clamp(0.0, 1.0);
            alpha = 3.0 * alpha * alpha - 2.0 * alpha * alpha * alpha;
            BlendingOutput {
                alpha,
                blending_offset: Self::height_to_offset(average_height),
            }
        }
    }

    fn height_to_offset(height: f64) -> f64 {
        let target_y = height + 0.5;
        let target_y_mod = target_y.rem_euclid(8.0);
        (32.0 * (target_y - 128.0) - 3.0 * (target_y - 120.0) * target_y_mod
            + 3.0 * target_y_mod * target_y_mod)
            / (128.0 * (32.0 - 3.0 * target_y_mod))
    }

    #[must_use]
    pub fn blend_density(&self, block_x: i32, block_y: i32, block_z: i32, noise_value: f64) -> f64 {
        let cell_x = biome_coords::from_block(block_x);
        let cell_y = block_y / 8;
        let cell_z = biome_coords::from_block(block_z);

        let fixed_density =
            self.get_blending_data_value(cell_x, cell_y, cell_z, |data, x, y, z| {
                data.get_density(x, y, z)
            });

        if fixed_density != f64::MAX {
            return fixed_density;
        }

        let mut total_weight = 0.0;
        let mut weighted_densities = 0.0;
        let mut closest_distance = f64::INFINITY;

        for (&packed_pos, blending_data) in &self.density_blending_data {
            let chunk_x = (packed_pos & 0xFFFFFFFF) as i32;
            let chunk_z = (packed_pos >> 32) as i32;

            blending_data.iterate_densities(
                biome_coords::from_chunk(chunk_x),
                biome_coords::from_chunk(chunk_z),
                cell_y - 1,
                cell_y + 1,
                |test_cell_x, test_cell_y, test_cell_z, density| {
                    let dx = (cell_x - test_cell_x) as f64;
                    let dy = ((cell_y - test_cell_y) * 2) as f64;
                    let dz = (cell_z - test_cell_z) as f64;
                    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

                    if distance <= 2.0 {
                        if distance < closest_distance {
                            closest_distance = distance;
                        }

                        let weight = 1.0 / (distance * distance * distance * distance);
                        weighted_densities += density * weight;
                        total_weight += weight;
                    }
                },
            );
        }

        if closest_distance == f64::INFINITY {
            noise_value
        } else {
            let average_density = weighted_densities / total_weight;
            let alpha = (closest_distance / 3.0).clamp(0.0, 1.0);
            alpha * noise_value + (1.0 - alpha) * average_density
        }
    }

    fn get_blending_data_value<F>(&self, cell_x: i32, cell_y: i32, cell_z: i32, getter: F) -> f64
    where
        F: Fn(&BlendingData, i32, i32, i32) -> f64,
    {
        let chunk_x = biome_coords::to_chunk(cell_x);
        let chunk_z = biome_coords::to_chunk(cell_z);
        let min_x = (cell_x & 3) == 0;
        let min_z = (cell_z & 3) == 0;

        let mut value = self.get_data_value(&getter, chunk_x, chunk_z, cell_x, cell_y, cell_z);
        if value == f64::MAX {
            if min_x && min_z {
                value =
                    self.get_data_value(&getter, chunk_x - 1, chunk_z - 1, cell_x, cell_y, cell_z);
            }

            if value == f64::MAX {
                if min_x {
                    value =
                        self.get_data_value(&getter, chunk_x - 1, chunk_z, cell_x, cell_y, cell_z);
                }

                if value == f64::MAX && min_z {
                    value =
                        self.get_data_value(&getter, chunk_x, chunk_z - 1, cell_x, cell_y, cell_z);
                }
            }
        }

        value
    }

    fn get_data_value<F>(
        &self,
        getter: &F,
        chunk_x: i32,
        chunk_z: i32,
        cell_x: i32,
        cell_y: i32,
        cell_z: i32,
    ) -> f64
    where
        F: Fn(&BlendingData, i32, i32, i32) -> f64,
    {
        let packed = (chunk_x as u32 as u64) | ((chunk_z as u32 as u64) << 32);
        if let Some(data) = self.height_and_biome_blending_data.get(&packed) {
            getter(
                data,
                cell_x - biome_coords::from_chunk(chunk_x),
                cell_y,
                cell_z - biome_coords::from_chunk(chunk_z),
            )
        } else {
            f64::MAX
        }
    }

    #[must_use]
    pub fn blend_biome(
        &self,
        quart_x: i32,
        quart_y: i32,
        quart_z: i32,
        shift_noise: &DoublePerlinNoiseSampler,
    ) -> Option<&'static Biome> {
        let mut closest_distance = f64::INFINITY;
        let mut closest_biome = None;

        for (&packed_pos, blending_data) in &self.height_and_biome_blending_data {
            let chunk_x = (packed_pos & 0xFFFFFFFF) as i32;
            let chunk_z = (packed_pos >> 32) as i32;

            blending_data.iterate_biomes(
                biome_coords::from_chunk(chunk_x),
                quart_y,
                biome_coords::from_chunk(chunk_z),
                |test_cell_x, test_cell_z, biome| {
                    let dx = (quart_x - test_cell_x) as f64;
                    let dz = (quart_z - test_cell_z) as f64;
                    let distance = dx.hypot(dz);

                    if distance <= HEIGHT_BLENDING_RANGE_CELLS as f64 && distance < closest_distance
                    {
                        closest_biome = Some(biome);
                        closest_distance = distance;
                    }
                },
            );
        }

        if closest_distance == f64::INFINITY {
            None
        } else {
            let shift = shift_noise.sample(quart_x as f64, 0.0, quart_z as f64) * 12.0;
            let alpha = ((closest_distance + shift) / (HEIGHT_BLENDING_RANGE_CELLS + 1) as f64)
                .clamp(0.0, 1.0);
            if alpha > 0.5 { None } else { closest_biome }
        }
    }
}

pub struct BlenderBiomeSupplier<'a> {
    base: &'a dyn BiomeSupplier,
    blender: &'a Blender,
    shift_noise: DoublePerlinNoiseSampler,
}

impl BiomeSupplier for BlenderBiomeSupplier<'_> {
    fn biome(&self, x: i32, y: i32, z: i32, sampler: &mut MultiNoiseSampler<'_>) -> &'static Biome {
        if let Some(blended) = self.blender.blend_biome(x, y, z, &self.shift_noise) {
            blended
        } else {
            self.base.biome(x, y, z, sampler)
        }
    }
}

pub trait BlenderImpl {
    fn blend_offset_and_factor(&self, block_x: i32, block_z: i32) -> BlendingOutput;

    fn blend_density(&self, pos: &Vector3<i32>, density: f64) -> f64;

    fn get_biome_supplier<'a>(
        &'a self,
        supplier: &'a dyn BiomeSupplier,
    ) -> BlenderBiomeSupplier<'a>;
}

impl BlenderImpl for Blender {
    fn blend_offset_and_factor(&self, block_x: i32, block_z: i32) -> BlendingOutput {
        self.blend_offset_and_factor(block_x, block_z)
    }

    fn blend_density(&self, pos: &Vector3<i32>, density: f64) -> f64 {
        self.blend_density(pos.x, pos.y, pos.z, density)
    }

    fn get_biome_supplier<'a>(
        &'a self,
        supplier: &'a dyn BiomeSupplier,
    ) -> BlenderBiomeSupplier<'a> {
        let mut random = Xoroshiro::from_seed(42);
        let shift_noise = DoublePerlinNoiseSampler::from_params(
            &mut random,
            &DoublePerlinNoiseParameters::OFFSET,
            false,
        );
        BlenderBiomeSupplier {
            base: supplier,
            blender: self,
            shift_noise,
        }
    }
}
