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
    /// Base noise column per (x, z), bottom-up from `shape.min_y`. The density pass
    /// that builds it dominates the cost, so one column serves every query about it.
    columns: FxHashMap<(i32, i32), Vec<&'static BlockState>>,
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
            columns: FxHashMap::default(),
        }
    }

    fn column(&mut self, x: i32, z: i32) -> &[&'static BlockState] {
        if !self.columns.contains_key(&(x, z)) {
            let column = self.sample_column(x, z);
            self.columns.insert((x, z), column);
        }
        &self.columns[&(x, z)]
    }

    fn sample_column(&mut self, x: i32, z: i32) -> Vec<&'static BlockState> {
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
        (0..volume.size_y)
            .map(|y| {
                let block_y = volume.block_y(y);
                let index = volume.index_unchecked(0, y, 0);
                noise
                    .sample_block_state(
                        &self.generator.random_config.ore_random_deriver,
                        &Vector3::new(x, block_y, z),
                        densities.density[index],
                        densities.vein_sample(index).as_ref(),
                        &mut self.preliminary,
                    )
                    .unwrap_or(self.generator.default_block)
            })
            .collect()
    }

    /// First Y from the top whose state passes `occupied`, plus one; `min_y` when the
    /// column is empty.
    fn top_of(&mut self, x: i32, z: i32, occupied: impl Fn(&BlockState) -> bool) -> i32 {
        let min_y = i32::from(self.generator.settings.shape.min_y);
        let column = self.column(x, z);
        column
            .iter()
            .rposition(|state| occupied(state))
            .map_or(min_y, |index| min_y + index as i32 + 1)
    }
}

impl HeightSampler for NoiseHeightSampler<'_> {
    fn estimate_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        self.top_of(block_x, block_z, |state| !state.is_air())
    }

    fn estimate_ocean_floor_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        // Vanilla's structure helper asks for the highest occupied block, while
        // heightmaps store the first free block above it.
        self.top_of(block_x, block_z, |state| {
            blocks_movement(state, BlockId::from_state_id(state.id))
        }) - 1
    }

    fn base_column_state(
        &mut self,
        block_x: i32,
        block_y: i32,
        block_z: i32,
    ) -> Option<&'static BlockState> {
        let min_y = i32::from(self.generator.settings.shape.min_y);
        let index = block_y - min_y;
        let column = self.column(block_x, block_z);
        Some(
            usize::try_from(index)
                .ok()
                .and_then(|index| column.get(index).copied())
                .unwrap_or(Block::AIR.default_state),
        )
    }
}
