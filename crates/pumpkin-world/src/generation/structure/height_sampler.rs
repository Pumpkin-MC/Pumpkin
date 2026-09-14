use pumpkin_data::{Block, BlockId, BlockState, block_properties::blocks_movement};
use pumpkin_util::math::vector3::Vector3;
use rustc_hash::FxHashMap;

use crate::generation::{
    generator::VanillaGenerator,
    noise::{
        ChunkNoiseGenerator,
        aquifer_sampler::FluidLevel,
        router::{
            density_volume::DensityVolume,
            surface_height_sampler::{
                SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
            },
        },
    },
    proto_chunk::StandardChunkFluidLevelSampler,
};

use super::structures::HeightSampler;

pub struct NoiseHeightSampler<'a> {
    generator: &'a VanillaGenerator,
    preliminary: SurfaceHeightEstimateSampler<'a>,
    heights: FxHashMap<(i32, i32), i32>,
    ocean_floor_heights: FxHashMap<(i32, i32), i32>,
    columns: FxHashMap<(i32, i32), (i32, Vec<&'static BlockState>)>,
}

impl<'a> NoiseHeightSampler<'a> {
    pub fn new(generator: &'a VanillaGenerator) -> Self {
        let shape = &generator.settings.shape;
        let preliminary = SurfaceHeightEstimateSampler::generate(
            &generator.base_router.surface_estimator,
            &SurfaceHeightSamplerBuilderOptions::new(
                i32::from(shape.min_y),
                i32::from(shape.max_y()),
                shape.vertical_cell_block_count() as usize,
            ),
        );
        Self {
            generator,
            preliminary,
            heights: FxHashMap::default(),
            ocean_floor_heights: FxHashMap::default(),
            columns: FxHashMap::default(),
        }
    }

    fn ensure_column_sampled(&mut self, x: i32, z: i32) {
        if self.columns.contains_key(&(x, z)) {
            return;
        }

        let settings = self.generator.settings;
        let shape = &settings.shape;
        let fluid_sampler = StandardChunkFluidLevelSampler::new(
            FluidLevel::new(
                settings.sea_level,
                Block::from_state_id(settings.default_fluid.id),
            ),
            FluidLevel::new(-54, &Block::LAVA),
        );
        let volume = DensityVolume::with_block_step(
            1,
            shape.height as usize,
            1,
            x,
            i32::from(shape.min_y),
            z,
        );
        let mut noise = ChunkNoiseGenerator::new(
            &self.generator.base_router.noise,
            &self.generator.random_config,
            volume,
            shape,
            fluid_sampler,
            settings.aquifers_enabled,
            false,
            Vec::new(),
            Vec::new(),
            None,
        );

        let densities = noise.sample_density();
        let mut blocks = Vec::with_capacity(volume.size_y);
        for y in 0..volume.size_y {
            let block_y = volume.block_y(y);
            let index = volume.index_unchecked(0, y, 0);
            let state = noise
                .sample_block_state(
                    &self.generator.random_config.ore_random_deriver,
                    &Vector3::new(x, block_y, z),
                    densities.density[index],
                    densities.vein_sample(index).as_ref(),
                    &mut self.preliminary,
                )
                .unwrap_or(self.generator.default_block);
            blocks.push(state);
        }
        self.columns
            .insert((x, z), (i32::from(shape.min_y), blocks));
    }

    fn sample_column(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.ensure_column_sampled(x, z);
        let Some((min_y, blocks)) = self.columns.get(&(x, z)) else {
            return 0;
        };
        let min_y = *min_y;
        for (idx, &state) in blocks.iter().enumerate().rev() {
            let block_y = min_y + idx as i32;
            if if ocean_floor {
                blocks_movement(state, BlockId::from_state_id(state.id))
            } else {
                !state.is_air()
            } {
                return block_y + 1;
            }
        }

        min_y
    }
}

impl HeightSampler for NoiseHeightSampler<'_> {
    fn estimate_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        let key = (block_x, block_z);
        if let Some(height) = self.heights.get(&key) {
            return *height;
        }
        let height = self.sample_column(block_x, block_z, false);
        self.heights.insert(key, height);
        height
    }

    fn estimate_ocean_floor_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        let key = (block_x, block_z);
        if let Some(height) = self.ocean_floor_heights.get(&key) {
            return *height;
        }
        // Vanilla's structure helper asks for the highest occupied block, while
        // heightmaps store the first free block above it.
        let height = self.sample_column(block_x, block_z, true) - 1;
        self.ocean_floor_heights.insert(key, height);
        height
    }

    fn sample_column_block(
        &mut self,
        block_x: i32,
        block_z: i32,
        y: i32,
    ) -> Option<&'static BlockState> {
        self.ensure_column_sampled(block_x, block_z);
        let (min_y, blocks) = self.columns.get(&(block_x, block_z))?;
        let idx = y - *min_y;
        if idx >= 0 && (idx as usize) < blocks.len() {
            Some(blocks[idx as usize])
        } else {
            Some(Block::AIR.default_state)
        }
    }
}
