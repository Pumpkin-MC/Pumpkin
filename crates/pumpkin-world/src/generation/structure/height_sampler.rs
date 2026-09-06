use pumpkin_data::{Block, BlockId, block_properties::blocks_movement};
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

/// One noise column, vanilla `NoiseBasedChunkGenerator.getBaseColumn`, reduced to the two
/// heightmap predicates that structure placement asks it about.
struct NoiseColumn {
    min_y: i32,
    not_air: Box<[bool]>,
    blocks_motion: Box<[bool]>,
}

impl NoiseColumn {
    fn top(&self, blocks_motion: bool) -> i32 {
        let flags = if blocks_motion {
            &self.blocks_motion
        } else {
            &self.not_air
        };
        for (index, solid) in flags.iter().enumerate().rev() {
            if *solid {
                return self.min_y + index as i32 + 1;
            }
        }
        self.min_y
    }

    fn is_opaque(&self, y: i32, blocks_motion: bool) -> bool {
        let index = y - self.min_y;
        if index < 0 || index >= self.not_air.len() as i32 {
            return false;
        }
        let flags = if blocks_motion {
            &self.blocks_motion
        } else {
            &self.not_air
        };
        flags[index as usize]
    }
}

pub struct NoiseHeightSampler<'a> {
    generator: &'a VanillaGenerator,
    preliminary: SurfaceHeightEstimateSampler<'a>,
    columns: FxHashMap<(i32, i32), NoiseColumn>,
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

    fn sample_column(&mut self, x: i32, z: i32) -> NoiseColumn {
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
        let mut not_air = vec![false; volume.size_y].into_boxed_slice();
        let mut blocks_motion = vec![false; volume.size_y].into_boxed_slice();
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
            not_air[y] = !state.is_air();
            blocks_motion[y] = blocks_movement(state, BlockId::from_state_id(state.id));
        }

        NoiseColumn {
            min_y: i32::from(shape.min_y),
            not_air,
            blocks_motion,
        }
    }

    fn column(&mut self, x: i32, z: i32) -> &NoiseColumn {
        if !self.columns.contains_key(&(x, z)) {
            let column = self.sample_column(x, z);
            self.columns.insert((x, z), column);
        }
        &self.columns[&(x, z)]
    }
}

impl HeightSampler for NoiseHeightSampler<'_> {
    fn estimate_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        self.column(block_x, block_z).top(false)
    }

    fn estimate_ocean_floor_height(&mut self, block_x: i32, block_z: i32) -> i32 {
        // Vanilla's structure helper asks for the highest occupied block, while
        // heightmaps store the first free block above it.
        self.column(block_x, block_z).top(true) - 1
    }

    fn column_is_opaque(&mut self, block_x: i32, block_z: i32, y: i32, ocean_floor: bool) -> bool {
        self.column(block_x, block_z).is_opaque(y, ocean_floor)
    }
}
