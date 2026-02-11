pub mod aquifer_sampler;
pub mod ore_sampler;
pub mod perlin;
pub mod router;

use pumpkin_data::{Block, BlockState, chunk_gen_settings::GenerationShapeConfig};
use pumpkin_util::math::{floor_div, floor_mod, vector3::Vector3};

use crate::generation::{
    noise::{
        aquifer_sampler::{
            AquiferSampler, AquiferSamplerImpl, FluidLevelSampler, SeaLevelAquiferSampler,
            WorldAquiferSampler,
        },
        ore_sampler::OreVeinSampler,
    },
    section_coords,
};

use super::{
    GlobalRandomConfig, biome_coords,
    noise::router::{
        chunk_density_function::{
            ChunkNoiseFunctionBuilderOptions, ChunkNoiseFunctionSampleOptions, SampleAction,
            WrapperData,
        },
        chunk_noise_router::ChunkNoiseRouter,
        density_function::IndexToNoisePos,
        proto_noise_router::ProtoNoiseRouter,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
};

pub const LAVA_BLOCK: Block = Block::LAVA;
pub const WATER_BLOCK: Block = Block::WATER;

pub const CHUNK_DIM: u8 = 16;

pub enum BlockStateSampler {
    Aquifer(AquiferSampler),
    Ore(OreVeinSampler),
}

impl BlockStateSampler {
    pub fn sample(
        &mut self,
        router: &mut ChunkNoiseRouter,
        random_config: &GlobalRandomConfig,
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        match self {
            Self::Aquifer(aquifer) => aquifer.apply(router, pos, sample_options, height_estimator),
            Self::Ore(ore) => ore.sample(router, random_config, pos, sample_options),
        }
    }
}

pub struct ChainedBlockStateSampler {
    pub(crate) samplers: Box<[BlockStateSampler]>,
}

impl ChainedBlockStateSampler {
    #[must_use]
    pub const fn new(samplers: Box<[BlockStateSampler]>) -> Self {
        Self { samplers }
    }

    fn sample(
        &mut self,
        router: &mut ChunkNoiseRouter,
        random_config: &GlobalRandomConfig,
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        for sampler in &mut self.samplers {
            if let Some(state) =
                sampler.sample(router, random_config, pos, sample_options, height_estimator)
            {
                return Some(state);
            }
        }
        None
    }
}

struct InterpolationIndexMapper {
    x: i32,
    z: i32,

    minimum_cell_y: i32,
    vertical_cell_block_count: i32,
}

impl IndexToNoisePos for InterpolationIndexMapper {
    fn at(
        &self,
        index: usize,
        sample_data: Option<&mut ChunkNoiseFunctionSampleOptions>,
    ) -> Vector3<i32> {
        if let Some(sample_data) = sample_data {
            sample_data.cache_result_unique_id += 1;
            sample_data.fill_index = index;
        }

        let y = (index as i32 + self.minimum_cell_y) * self.vertical_cell_block_count;

        // TODO: Change this when Blender is implemented
        Vector3::new(self.x, y, self.z)
    }
}

struct ChunkIndexMapper {
    start_x: i32,
    start_y: i32,
    start_z: i32,

    horizontal_cell_block_count: usize,
    vertical_cell_block_count: usize,
}

impl IndexToNoisePos for ChunkIndexMapper {
    fn at(
        &self,
        index: usize,
        sample_options: Option<&mut ChunkNoiseFunctionSampleOptions>,
    ) -> Vector3<i32> {
        let cell_z_position = floor_mod(index, self.horizontal_cell_block_count);
        let xy_portion = floor_div(index, self.horizontal_cell_block_count);
        let cell_x_position = floor_mod(xy_portion, self.horizontal_cell_block_count);
        let cell_y_position = self.vertical_cell_block_count
            - 1
            - floor_div(xy_portion, self.horizontal_cell_block_count);

        if let Some(sample_options) = sample_options {
            sample_options.fill_index = index;
            if let SampleAction::CellCaches(wrapper_data) = &mut sample_options.action {
                wrapper_data.update_position(cell_x_position, cell_y_position, cell_z_position);
            }
        }

        // TODO: Change this when Blender is implemented
        Vector3::new(
            self.start_x + cell_x_position as i32,
            self.start_y + cell_y_position as i32,
            self.start_z + cell_z_position as i32,
        )
    }
}

pub struct ChunkNoiseGenerator<'a> {
    pub state_sampler: ChainedBlockStateSampler,
    generation_shape: &'a GenerationShapeConfig,
    start_cell_pos_x: i32,
    start_cell_pos_z: i32,

    vertical_cell_count: usize,
    minimum_cell_y: i32,

    cache_fill_unique_id: u64,
    cache_result_unique_id: u64,

    pub router: ChunkNoiseRouter<'a>,
}

impl<'a> ChunkNoiseGenerator<'a> {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        noise_router_base: &'a ProtoNoiseRouter,
        random_config: &GlobalRandomConfig,
        horizontal_cell_count: usize,
        start_block_x: i32,
        start_block_z: i32,
        generation_shape: &'a GenerationShapeConfig,
        level_sampler: FluidLevelSampler,
        aquifers: bool,
        ore_veins: bool,
    ) -> Self {
        let start_cell_pos_x = floor_div(
            start_block_x,
            generation_shape.horizontal_cell_block_count() as i32,
        );
        let start_cell_pos_z = floor_div(
            start_block_z,
            generation_shape.horizontal_cell_block_count() as i32,
        );

        let horizontal_biome_end = biome_coords::from_block(
            horizontal_cell_count as i32 * generation_shape.horizontal_cell_block_count() as i32,
        );
        let vertical_cell_count = floor_div(
            generation_shape.height as usize,
            generation_shape.vertical_cell_block_count() as usize,
        );
        let minimum_cell_y = floor_div(
            generation_shape.min_y as i32,
            generation_shape.vertical_cell_block_count() as i32,
        );
        let vertical_cell_block_count = generation_shape.vertical_cell_block_count();
        let horizontal_cell_block_count = generation_shape.horizontal_cell_block_count();

        let builder_options = ChunkNoiseFunctionBuilderOptions::new(
            horizontal_cell_block_count as usize,
            vertical_cell_block_count as usize,
            vertical_cell_count,
            horizontal_cell_count,
            biome_coords::from_block(start_block_x),
            biome_coords::from_block(start_block_z),
            horizontal_biome_end as usize,
        );

        let aquifer_sampler = if aquifers {
            let section_x = section_coords::block_to_section(start_block_x);
            let section_z = section_coords::block_to_section(start_block_z);
            AquiferSampler::Aquifer(WorldAquiferSampler::new(
                section_x,
                section_z,
                &random_config.aquifer_random_deriver,
                generation_shape.min_y,
                generation_shape.height,
                level_sampler,
            ))
        } else {
            AquiferSampler::SeaLevel(SeaLevelAquiferSampler::new(level_sampler))
        };

        let samplers: Box<[BlockStateSampler]> = if ore_veins {
            Box::new([
                BlockStateSampler::Aquifer(aquifer_sampler),
                BlockStateSampler::Ore(OreVeinSampler),
            ])
        } else {
            Box::new([BlockStateSampler::Aquifer(aquifer_sampler)])
        };
        let state_sampler = ChainedBlockStateSampler::new(samplers);

        let router = ChunkNoiseRouter::generate(noise_router_base, &builder_options);

        Self {
            state_sampler,
            generation_shape,
            start_cell_pos_x,
            start_cell_pos_z,
            vertical_cell_count,
            minimum_cell_y,

            cache_fill_unique_id: 0,
            cache_result_unique_id: 0,

            router,
        }
    }

    #[inline]
    pub fn sample_start_density(&mut self) {
        self.cache_result_unique_id = 0;
        self.sample_density(true, self.start_cell_pos_x);
    }

    #[inline]
    pub fn sample_end_density(&mut self, cell_x: i32) {
        self.sample_density(false, self.start_cell_pos_x + cell_x + 1);
    }

    fn sample_density(&mut self, start: bool, current_x: i32) {
        let x = current_x * self.horizontal_cell_block_count() as i32;

        for cell_z in 0..=(16 / self.horizontal_cell_block_count()) {
            let current_cell_z_pos = self.start_cell_pos_z + cell_z as i32;
            let z = current_cell_z_pos * self.horizontal_cell_block_count() as i32;
            self.cache_fill_unique_id += 1;

            let mapper = InterpolationIndexMapper {
                x,
                z,
                minimum_cell_y: self.minimum_cell_y,
                vertical_cell_block_count: self.vertical_cell_block_count() as i32,
            };

            let mut options = ChunkNoiseFunctionSampleOptions::new(
                false,
                SampleAction::CellCaches(WrapperData::new(
                    0,
                    0,
                    0,
                    self.horizontal_cell_block_count() as usize,
                    self.vertical_cell_block_count() as usize,
                )),
                self.cache_result_unique_id,
                self.cache_fill_unique_id,
                0,
            );

            self.fill_interpolator_buffers(start, cell_z as usize, &mapper, &mut options);
            self.cache_result_unique_id = options.cache_result_unique_id;
        }
        self.cache_fill_unique_id += 1;
    }

    #[inline]
    fn fill_interpolator_buffers(
        &mut self,
        start: bool,
        cell_z: usize,
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        self.router
            .fill_interpolator_buffers(start, cell_z, mapper, sample_options);
    }

    #[inline]
    pub fn interpolate_x(&mut self, delta: f64) {
        self.router.interpolate_x(delta);
    }

    #[inline]
    pub fn interpolate_y(&mut self, delta: f64) {
        self.router.interpolate_y(delta);
    }

    #[inline]
    pub fn interpolate_z(&mut self, delta: f64) {
        self.cache_result_unique_id += 1;
        self.router.interpolate_z(delta);
    }

    #[inline]
    pub fn swap_buffers(&mut self) {
        self.router.swap_buffers();
    }

    pub fn on_sampled_cell_corners(&mut self, cell_x: i32, cell_y: i32, cell_z: i32) {
        self.router
            .on_sampled_cell_corners(cell_y as usize, cell_z as usize);
        self.cache_fill_unique_id += 1;

        let start_x = (self.start_cell_pos_x + cell_x) * self.horizontal_cell_block_count() as i32;
        let start_y = (cell_y + self.minimum_cell_y) * self.vertical_cell_block_count() as i32;
        let start_z = (self.start_cell_pos_z + cell_z) * self.horizontal_cell_block_count() as i32;

        let mapper = ChunkIndexMapper {
            start_x,
            start_y,
            start_z,
            horizontal_cell_block_count: self.horizontal_cell_block_count() as usize,
            vertical_cell_block_count: self.vertical_cell_block_count() as usize,
        };

        let mut sample_options = ChunkNoiseFunctionSampleOptions::new(
            true,
            SampleAction::CellCaches(WrapperData::new(
                0,
                0,
                0,
                self.horizontal_cell_block_count() as usize,
                self.vertical_cell_block_count() as usize,
            )),
            self.cache_result_unique_id,
            self.cache_fill_unique_id,
            0,
        );

        self.router.fill_cell_caches(&mapper, &mut sample_options);
        self.cache_fill_unique_id += 1;
    }

    #[expect(clippy::too_many_arguments)]
    pub fn sample_block_state(
        &mut self,
        random_config: &GlobalRandomConfig,
        start_x: i32,
        start_y: i32,
        start_z: i32,
        cell_x: i32,
        cell_y: i32,
        cell_z: i32,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        //TODO: Fix this when Blender is added
        let pos = Vector3::new(start_x + cell_x, start_y + cell_y, start_z + cell_z);

        let options = ChunkNoiseFunctionSampleOptions::new(
            false,
            SampleAction::CellCaches(WrapperData::new(
                cell_x as usize,
                cell_y as usize,
                cell_z as usize,
                self.horizontal_cell_block_count() as usize,
                self.vertical_cell_block_count() as usize,
            )),
            self.cache_result_unique_id,
            self.cache_fill_unique_id,
            0,
        );

        self.state_sampler.sample(
            &mut self.router,
            random_config,
            &pos,
            &options,
            height_estimator,
        )
    }

    #[inline]
    #[must_use]
    pub fn horizontal_cell_block_count(&self) -> u8 {
        self.generation_shape.horizontal_cell_block_count()
    }

    #[inline]
    #[must_use]
    pub fn vertical_cell_block_count(&self) -> u8 {
        self.generation_shape.vertical_cell_block_count()
    }

    /// Aka `bottom_y`
    #[inline]
    #[must_use]
    pub const fn min_y(&self) -> i8 {
        self.generation_shape.min_y
    }

    #[inline]
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.generation_shape.height
    }
}

#[cfg(test)]
mod test {
    // TODO: Add test to verify the height estimator has no interpolators or cell caches
}
