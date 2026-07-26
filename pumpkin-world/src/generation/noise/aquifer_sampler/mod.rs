use enum_dispatch::enum_dispatch;
use pumpkin_data::{Block, BlockState, chunk_gen_settings::GenerationSettings};
use pumpkin_util::{
    math::{clamped_map, floor_div, vector3::Vector3},
    random::{RandomImpl, xoroshiro128::XoroshiroSplitter},
};

use crate::generation::{
    GlobalRandomConfig, biome_coords,
    noise::{
        CHUNK_DIM, LAVA_BLOCK, WATER_BLOCK,
        router::{
            chunk_density_function::{
                ChunkNoiseFunctionBuilderOptions, ChunkNoiseFunctionSampleOptions, SampleAction,
            },
            chunk_noise_router::ChunkNoiseRouter,
            proto_noise_router::ProtoNoiseRouters,
            surface_height_sampler::SurfaceHeightEstimateSampler,
        },
    },
    positions::{MIN_HEIGHT_CELL, block_pos, chunk_pos},
    proto_chunk::StandardChunkFluidLevelSampler,
    section_coords,
};

#[derive(Clone)]
pub struct FluidLevel {
    max_y: i32,
    block: &'static Block,
}

impl FluidLevel {
    #[must_use]
    pub const fn new(max_y: i32, block: &'static Block) -> Self {
        Self { max_y, block }
    }

    #[must_use]
    pub const fn max_y_exclusive(&self) -> i32 {
        self.max_y
    }

    const fn get_block(&self, y: i32) -> &'static Block {
        if y < self.max_y {
            self.block
        } else {
            &Block::AIR
        }
    }
}

pub trait FluidLevelSamplerImpl {
    fn get_fluid_level(&self, x: i32, y: i32, z: i32) -> &FluidLevel;
}

#[enum_dispatch(AquiferSamplerImpl)]
pub enum AquiferSampler {
    SeaLevel(SeaLevelAquiferSampler),
    Aquifer(WorldAquiferSampler),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarverAquiferResult {
    pub state: Option<&'static BlockState>,
    pub should_schedule_fluid_update: bool,
}

pub struct CarverAquiferSampler<'a> {
    aquifer: WorldAquiferSampler,
    router: ChunkNoiseRouter<'a>,
    height_estimator: SurfaceHeightEstimateSampler<'a>,
    sample_options: ChunkNoiseFunctionSampleOptions,
}

impl<'a> CarverAquiferSampler<'a> {
    #[must_use]
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        base_router: &'a ProtoNoiseRouters,
        random_config: &GlobalRandomConfig,
        settings: &GenerationSettings,
    ) -> Self {
        let shape = &settings.shape;
        let horizontal_cell_count = CHUNK_DIM / shape.horizontal_cell_block_count();
        let start_x = chunk_pos::start_block_x(chunk_x);
        let start_z = chunk_pos::start_block_z(chunk_z);
        let horizontal_biome_end = biome_coords::from_block(
            horizontal_cell_count as i32 * shape.horizontal_cell_block_count() as i32,
        );
        let builder_options = ChunkNoiseFunctionBuilderOptions::new(
            shape.horizontal_cell_block_count() as usize,
            shape.vertical_cell_block_count() as usize,
            floor_div(
                shape.height as usize,
                shape.vertical_cell_block_count() as usize,
            ),
            horizontal_cell_count as usize,
            biome_coords::from_block(start_x),
            biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
            Vec::new(),
            Vec::new(),
            None,
        );
        let surface_config =
            super::router::surface_height_sampler::SurfaceHeightSamplerBuilderOptions::new(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_z),
                horizontal_biome_end as usize,
                shape.min_y as i32,
                shape.max_y() as i32,
                shape.vertical_cell_block_count() as usize,
            );
        let fluid_level = StandardChunkFluidLevelSampler::new(
            FluidLevel::new(
                settings.sea_level,
                Block::from_state_id(settings.default_fluid.id),
            ),
            FluidLevel::new(-54, &Block::LAVA),
        );

        Self {
            aquifer: WorldAquiferSampler::new(
                chunk_x,
                chunk_z,
                &random_config.aquifer_random_deriver,
                shape.min_y,
                shape.height,
                fluid_level,
            ),
            router: ChunkNoiseRouter::generate(&base_router.noise, &builder_options),
            height_estimator: SurfaceHeightEstimateSampler::generate(
                &base_router.surface_estimator,
                &surface_config,
            ),
            sample_options: ChunkNoiseFunctionSampleOptions::new(
                false,
                SampleAction::SkipCellCaches,
                0,
                0,
                0,
            ),
        }
    }

    pub fn compute(&mut self, pos: &Vector3<i32>, density: f64) -> CarverAquiferResult {
        let (state, should_schedule_fluid_update) = self.aquifer.apply_internal(
            &mut self.router,
            pos,
            &self.sample_options,
            &mut self.height_estimator,
            density,
        );

        CarverAquiferResult {
            state,
            should_schedule_fluid_update,
        }
    }
}

macro_rules! packed_position_index {
    ($local_x:expr,$local_y:expr,$local_z:expr,$dim_y:expr,$dim_z:expr) => {
        ($local_x * $dim_z + $local_z) * $dim_y + $local_y
    };
}

macro_rules! local_xz {
    ($xz:expr) => {
        floor_div($xz, 16)
    };
}

macro_rules! local_y {
    ($y:expr) => {
        floor_div($y, 12)
    };
}

pub struct WorldAquiferSampler {
    fluid_level_sampler: StandardChunkFluidLevelSampler,
    start_x: i32,
    start_y: i32,
    start_z: i32,
    size_y: usize,
    size_z: usize,
    levels: Box<[Option<FluidLevel>]>,
    packed_positions: Box<[i64]>,
}

impl WorldAquiferSampler {
    const CHUNK_POS_OFFSETS: [(i8, i8); 13] = [
        (0, 0),
        (-2, -1),
        (-1, -1),
        (0, -1),
        (1, -1),
        (-3, 0),
        (-2, 0),
        (-1, 0),
        (1, 0),
        (-2, 1),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    #[must_use]
    pub fn new(
        chunk_x: i32,
        chunk_z: i32,
        random_deriver: &XoroshiroSplitter,
        minimum_y: i8,
        height: u16,
        fluid_level: StandardChunkFluidLevelSampler,
    ) -> Self {
        let start_x = local_xz!(chunk_pos::start_block_x(chunk_x)) - 1;
        let end_x = local_xz!(chunk_pos::end_block_x(chunk_x)) + 1;
        let size_x = (end_x - start_x) as usize + 1;

        let start_y = local_y!(minimum_y) - 1;
        let end_y = local_y!(minimum_y as i32 + height as i32) + 1;
        let size_y = (end_y - start_y as i32) as usize + 1;

        let start_z = local_xz!(chunk_pos::start_block_z(chunk_z)) - 1;
        let end_z = local_xz!(chunk_pos::end_block_z(chunk_z)) + 1;
        let size_z = (end_z - start_z) as usize + 1;

        let cache_size = size_x * size_y * size_z;

        let mut packed_positions = vec![0; cache_size];

        for offset_x in 0..size_x {
            for offset_y in 0..size_y {
                for offset_z in 0..size_z {
                    let x = start_x + offset_x as i32;
                    let y = start_y as i32 + offset_y as i32;
                    let z = start_z + offset_z as i32;

                    let mut random = random_deriver.split_pos(x, y, z);
                    let rand_x = x * 16 + random.next_bounded_i32(10);
                    let rand_y = y * 12 + random.next_bounded_i32(9);
                    let rand_z = z * 16 + random.next_bounded_i32(10);

                    let index =
                        packed_position_index!(offset_x, offset_y, offset_z, size_y, size_z);
                    packed_positions[index as usize] =
                        block_pos::packed(rand_x as i64, rand_y as i64, rand_z as i64);
                }
            }
        }

        Self {
            fluid_level_sampler: fluid_level,
            start_x,
            start_y: start_y as i32,
            start_z,
            size_y,
            size_z,
            levels: vec![None; cache_size as usize].into(),
            packed_positions: packed_positions.into(),
        }
    }

    fn checked_packed_position_index(&self, x: i32, y: i32, z: i32) -> Option<usize> {
        let local_x = usize::try_from(x - self.start_x).ok()?;
        let local_y = usize::try_from(y - self.start_y).ok()?;
        let local_z = usize::try_from(z - self.start_z).ok()?;

        if local_y >= self.size_y || local_z >= self.size_z {
            return None;
        }

        let index = packed_position_index!(local_x, local_y, local_z, self.size_y, self.size_z);
        (index < self.packed_positions.len()).then_some(index)
    }

    fn random_positions_for_pos(&self, x: i32, y: i32, z: i32) -> Option<[i64; 12]> {
        let sy = self.size_y;
        let syz = self.size_y * self.size_z;

        let i00 = self.checked_packed_position_index(x, y - 1, z)?;
        let i01 = i00 + sy;
        let i10 = i00 + syz;
        let i11 = i10 + sy;

        if i11 + 2 >= self.packed_positions.len() {
            return None;
        }

        let p = &self.packed_positions;

        Some([
            p[i11 + 2],
            p[i10 + 2],
            p[i01 + 2],
            p[i00 + 2],
            p[i11 + 1],
            p[i10 + 1],
            p[i01 + 1],
            p[i00 + 1],
            p[i11],
            p[i10],
            p[i01],
            p[i00],
        ])
    }

    #[inline]
    fn max_distance(i: i32, a: i32) -> f64 {
        1f64 - ((a - i).abs() as f64) / 25f64
    }

    fn calculate_density(
        barrier_sample: &mut Option<f64>,
        pos: &Vector3<i32>,
        router: &mut ChunkNoiseRouter,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        level_1: &FluidLevel,
        level_2: &FluidLevel,
    ) -> f64 {
        let y = pos.y;
        let block_state1 = level_1.get_block(y);
        let block_state2 = level_2.get_block(y);

        if (block_state1 != &LAVA_BLOCK || block_state2 != &WATER_BLOCK)
            && (block_state1 != &WATER_BLOCK || block_state2 != &LAVA_BLOCK)
        {
            let level_diff = (level_1.max_y - level_2.max_y).abs();
            if level_diff == 0 {
                0f64
            } else {
                let avg_level = 0.5f64 * (level_1.max_y + level_2.max_y) as f64;
                let scaled_level = y as f64 + 0.5f64 - avg_level;
                let halved_diff = level_diff as f64 / 2f64;

                let o = halved_diff - scaled_level.abs();
                let q = if scaled_level > 0f64 {
                    if o > 0f64 { o / 1.5f64 } else { o / 2.5f64 }
                } else {
                    let p = 3f64 + o;
                    if p > 0f64 { p / 3f64 } else { p / 10f64 }
                };

                let r = if (-2f64..=2f64).contains(&q) {
                    *barrier_sample.get_or_insert_with(|| router.barrier_noise(pos, sample_options))
                } else {
                    0f64
                };

                2f64 * (r + q)
            }
        } else {
            2f64
        }
    }

    fn get_water_level(
        &mut self,
        packed_pos: i64,
        router: &mut ChunkNoiseRouter,
        height_estimator: &mut SurfaceHeightEstimateSampler,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> FluidLevel {
        let x = block_pos::unpack_x(packed_pos);
        let y = block_pos::unpack_y(packed_pos);
        let z = block_pos::unpack_z(packed_pos);

        let local_x = local_xz!(x);
        let local_y = local_y!(y);
        let local_z = local_xz!(z);

        let Some(index) = self.checked_packed_position_index(local_x, local_y, local_z) else {
            return Self::get_fluid_level(
                &self.fluid_level_sampler,
                x,
                y,
                z,
                router,
                height_estimator,
                sample_options,
            );
        };

        if let Some(ref level) = self.levels[index] {
            return level.clone();
        }

        let sampled = Self::get_fluid_level(
            &self.fluid_level_sampler,
            x,
            y,
            z,
            router,
            height_estimator,
            sample_options,
        );

        self.levels[index] = Some(sampled.clone());
        sampled
    }

    fn get_fluid_level(
        fluid_level_sampler: &StandardChunkFluidLevelSampler,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        router: &mut ChunkNoiseRouter,
        height_estimator: &mut SurfaceHeightEstimateSampler,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> FluidLevel {
        let fluid_level = fluid_level_sampler.get_fluid_level(block_x, block_y, block_z);
        let j = block_y + 12;
        let k = block_y - 12;
        let mut bl = false;
        let mut min_surface_estimate = i32::MAX;

        for (offset_x, offset_z) in Self::CHUNK_POS_OFFSETS {
            let x = block_x + section_coords::section_to_block(offset_x as i32);
            let z = block_z + section_coords::section_to_block(offset_z as i32);

            let n = height_estimator.estimate_height(x, z);
            let o = n + 8;
            let bl2 = offset_x == 0 && offset_z == 0;

            if bl2 && k > o {
                return fluid_level.clone();
            }

            let bl3 = j > o;
            if bl3 || bl2 {
                let fluid_level = fluid_level_sampler.get_fluid_level(x, o, z);
                if !fluid_level.get_block(o).default_state.is_air() {
                    if bl2 {
                        bl = true;
                    }

                    if bl3 {
                        return fluid_level.clone();
                    }
                }
            }

            min_surface_estimate = min_surface_estimate.min(n);
        }

        let p = Self::get_fluid_block_y(
            block_x,
            block_y,
            block_z,
            fluid_level,
            min_surface_estimate,
            bl,
            router,
            sample_options,
        );
        FluidLevel::new(
            p,
            Self::get_fluid_block_state(
                block_x,
                block_y,
                block_z,
                fluid_level,
                p,
                router,
                sample_options,
            ),
        )
    }

    #[expect(clippy::too_many_arguments)]
    fn get_fluid_block_y(
        block_x: i32,
        block_y: i32,
        block_z: i32,
        default_level: &FluidLevel,
        surface_height_estimate: i32,
        map_y: bool,
        router: &mut ChunkNoiseRouter,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> i32 {
        let pos = Vector3::new(block_x, block_y, block_z);

        let is_deep_dark = router.erosion(&pos, sample_options) < -0.225f32 as f64
            && router.depth(&pos, sample_options) > 0.9f32 as f64;

        let (d, e) = if is_deep_dark {
            (-1f64, -1f64)
        } else {
            let top_y = surface_height_estimate + 8 - block_y;
            let f = if map_y {
                clamped_map(top_y as f64, 0f64, 64f64, 1f64, 0f64)
            } else {
                0f64
            };

            let g = router
                .fluid_level_floodedness_noise(&pos, sample_options)
                .clamp(-1f64, 1f64);
            let h = pumpkin_util::math::map(f, 1f64, 0f64, -0.3f64, 0.8f64);
            let k = pumpkin_util::math::map(f, 1f64, 0f64, -0.8f64, 0.4f64);

            (g - k, g - h)
        };

        if e > 0f64 {
            default_level.max_y
        } else if d > 0f64 {
            Self::get_noise_based_fluid_level(
                block_x,
                block_y,
                block_z,
                surface_height_estimate,
                router,
                sample_options,
            )
        } else {
            MIN_HEIGHT_CELL
        }
    }

    fn get_noise_based_fluid_level(
        block_x: i32,
        block_y: i32,
        block_z: i32,
        surface_height_estimate: i32,
        router: &mut ChunkNoiseRouter,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> i32 {
        let x = floor_div(block_x, 16);
        let y = floor_div(block_y, 40);
        let z = floor_div(block_z, 16);

        let local_y = y * 40 + 20;

        let sample =
            router.fluid_level_spread_noise(&Vector3::new(x, y, z), sample_options) * 10f64;
        let to_nearest_multiple_of_three = (sample / 3f64).floor() as i32 * 3;
        let local_height = to_nearest_multiple_of_three + local_y;

        surface_height_estimate.min(local_height)
    }

    fn get_fluid_block_state(
        block_x: i32,
        block_y: i32,
        block_z: i32,
        default_level: &FluidLevel,
        level: i32,
        router: &mut ChunkNoiseRouter,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> &'static Block {
        if level <= -10 && level != MIN_HEIGHT_CELL && default_level.block != &LAVA_BLOCK {
            let x = floor_div(block_x, 64);
            let y = floor_div(block_y, 40);
            let z = floor_div(block_z, 64);

            let sample = router.lava_noise(&Vector3::new(x, y, z), sample_options);

            if sample.abs() > 0.3f64 {
                return &LAVA_BLOCK;
            }
        }

        default_level.block
    }

    #[expect(clippy::too_many_lines)]
    fn apply_internal(
        &mut self,
        router: &mut ChunkNoiseRouter,
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        height_estimator: &mut SurfaceHeightEstimateSampler,
        density: f64,
    ) -> (Option<&'static BlockState>, bool) {
        if density > 0f64 {
            return (None, false);
        }

        let sample_x = pos.x;
        let sample_y = pos.y;
        let sample_z = pos.z;

        let fluid_level = self
            .fluid_level_sampler
            .get_fluid_level(sample_x, sample_y, sample_z);
        if fluid_level.get_block(sample_y) == &LAVA_BLOCK {
            return (Some(LAVA_BLOCK.default_state), false);
        }

        let scaled_x = local_xz!(sample_x - 5);
        let scaled_y = local_y!(sample_y + 1);
        let scaled_z = local_xz!(sample_z - 5);

        let Some(random_positions) = self.random_positions_for_pos(scaled_x, scaled_y, scaled_z)
        else {
            return (Some(fluid_level.get_block(sample_y).default_state), false);
        };

        let mut nearest = [(0i64, i32::MAX); 4];

        macro_rules! process {
            ($packed:expr) => {{
                let packed = $packed;
                let dx = block_pos::unpack_x(packed) - sample_x;
                let dy = block_pos::unpack_y(packed) - sample_y;
                let dz = block_pos::unpack_z(packed) - sample_z;
                let h = dx * dx + dy * dy + dz * dz;
                if nearest[3].1 > h {
                    nearest[3] = (packed, h);
                }
                if nearest[2].1 > h {
                    nearest[3] = nearest[2];
                    nearest[2] = (packed, h);
                }
                if nearest[1].1 > h {
                    nearest[2] = nearest[1];
                    nearest[1] = (packed, h);
                }
                if nearest[0].1 > h {
                    nearest[1] = nearest[0];
                    nearest[0] = (packed, h);
                }
            }};
        }

        // Same insertion order as the original array literal; sort behaviour is preserved.
        for packed in random_positions {
            process!(packed);
        }

        let fluid_level2 =
            self.get_water_level(nearest[0].0, router, height_estimator, sample_options);
        let block_state = fluid_level2.get_block(sample_y);
        let sim12 = Self::max_distance(nearest[0].1, nearest[1].1);

        if sim12 <= 0f64 {
            let should_schedule = if sim12 >= -0.12f64 {
                // FLOWING_UPDATE_SIMILARITY
                let fluid_level3 =
                    self.get_water_level(nearest[1].0, router, height_estimator, sample_options);
                fluid_level2.block != fluid_level3.block || fluid_level2.max_y != fluid_level3.max_y
            } else {
                false
            };
            return (Some(block_state.default_state), should_schedule);
        }

        if block_state == &WATER_BLOCK
            && self
                .fluid_level_sampler
                .get_fluid_level(sample_x, sample_y - 1, sample_z)
                .get_block(sample_y - 1)
                == &LAVA_BLOCK
        {
            return (Some(block_state.default_state), true);
        }

        let mut barrier_sample = None;
        let fluid_level3 =
            self.get_water_level(nearest[1].0, router, height_estimator, sample_options);
        let barrier12 = sim12
            * Self::calculate_density(
                &mut barrier_sample,
                pos,
                router,
                sample_options,
                &fluid_level2,
                &fluid_level3,
            );

        if density + barrier12 > 0f64 {
            return (None, false);
        }

        let fluid_level4 =
            self.get_water_level(nearest[2].0, router, height_estimator, sample_options);
        let sim13 = Self::max_distance(nearest[0].1, nearest[2].1);
        if sim13 > 0f64 {
            let barrier13 = sim12
                * sim13
                * Self::calculate_density(
                    &mut barrier_sample,
                    pos,
                    router,
                    sample_options,
                    &fluid_level2,
                    &fluid_level4,
                );
            if density + barrier13 > 0f64 {
                return (None, false);
            }
        }

        let sim23 = Self::max_distance(nearest[1].1, nearest[2].1);
        if sim23 > 0f64 {
            let barrier23 = sim12
                * sim23
                * Self::calculate_density(
                    &mut barrier_sample,
                    pos,
                    router,
                    sample_options,
                    &fluid_level3,
                    &fluid_level4,
                );
            if density + barrier23 > 0f64 {
                return (None, false);
            }
        }

        let may_flow12 =
            fluid_level2.block != fluid_level3.block || fluid_level2.max_y != fluid_level3.max_y;
        let may_flow23 = sim23 >= -0.12f64
            && (fluid_level3.block != fluid_level4.block
                || fluid_level3.max_y != fluid_level4.max_y);
        let may_flow13 = sim13 >= -0.12f64
            && (fluid_level2.block != fluid_level4.block
                || fluid_level2.max_y != fluid_level4.max_y);

        let should_schedule = if may_flow12 || may_flow23 || may_flow13 {
            true
        } else {
            let fluid_level5 =
                self.get_water_level(nearest[3].0, router, height_estimator, sample_options);
            sim13 >= -0.12f64
                && Self::max_distance(nearest[0].1, nearest[3].1) >= -0.12f64
                && (fluid_level2.block != fluid_level5.block
                    || fluid_level2.max_y != fluid_level5.max_y)
        };

        (Some(block_state.default_state), should_schedule)
    }
}

impl AquiferSamplerImpl for WorldAquiferSampler {
    #[inline]
    fn apply(
        &mut self,
        router: &mut ChunkNoiseRouter,
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> (Option<&'static BlockState>, bool) {
        let density = router.final_density(pos, sample_options);
        self.apply_internal(router, pos, sample_options, height_estimator, density)
    }
}

pub struct SeaLevelAquiferSampler {
    level_sampler: StandardChunkFluidLevelSampler,
}

impl SeaLevelAquiferSampler {
    #[must_use]
    pub const fn new(level_sampler: StandardChunkFluidLevelSampler) -> Self {
        Self { level_sampler }
    }
}

impl AquiferSamplerImpl for SeaLevelAquiferSampler {
    fn apply(
        &mut self,
        router: &mut ChunkNoiseRouter,
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        _height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> (Option<&'static BlockState>, bool) {
        let sample = router.final_density(pos, sample_options);
        if sample > 0f64 {
            (None, false)
        } else {
            (
                Some(
                    self.level_sampler
                        .get_fluid_level(pos.x, pos.y, pos.z)
                        .get_block(pos.y)
                        .default_state,
                ),
                false,
            )
        }
    }
}

#[enum_dispatch]
pub trait AquiferSamplerImpl {
    fn apply(
        &mut self,
        router: &mut ChunkNoiseRouter,
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> (Option<&'static BlockState>, bool);
}

#[cfg(test)]
mod random_positions_and_hypot;
