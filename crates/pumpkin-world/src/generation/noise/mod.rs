pub mod aquifer_sampler;
pub mod ore_sampler;
pub mod perlin;
pub mod router;

use pumpkin_data::{Block, BlockState, noise_settings::GenerationShapeConfig};
use pumpkin_util::{math::vector3::Vector3, random::xoroshiro128::XoroshiroSplitter};

use crate::generation::{
    noise::{
        aquifer_sampler::{
            AquiferSampler, AquiferSamplerImpl, SeaLevelAquiferSampler, WorldAquiferSampler,
        },
        ore_sampler::OreVeinSampler,
    },
    proto_chunk::StandardChunkFluidLevelSampler,
    section_coords,
};

use super::{
    GlobalRandomConfig,
    noise::router::{
        chunk_density_function::ChunkNoiseFunctionBuilderOptions,
        chunk_noise_router::ChunkNoiseRouter,
        density_volume::{DensityBuffer, DensityVolume},
        proto_noise_router::ProtoNoiseRouter,
        surface_height_sampler::SurfaceHeightEstimateSampler,
    },
};

pub const LAVA_BLOCK: Block = Block::LAVA;
pub const WATER_BLOCK: Block = Block::WATER;

pub const CHUNK_DIM: u8 = 16;

pub struct VeinSample {
    pub toggle: f32,
    pub ridged: f32,
}

pub struct ChunkDensities {
    pub density: DensityBuffer,
    veins: Option<[DensityBuffer; 2]>,
}

impl ChunkDensities {
    #[must_use]
    pub fn vein_sample(&self, index: usize) -> Option<VeinSample> {
        self.veins.as_ref().map(|[toggle, ridged]| VeinSample {
            toggle: toggle[index],
            ridged: ridged[index],
        })
    }
}

pub enum BlockStateSampler {
    Aquifer(AquiferSampler),
    Ore(OreVeinSampler),
}

impl BlockStateSampler {
    pub fn sample(
        &mut self,
        router: &mut ChunkNoiseRouter,
        ore_random_deriver: &XoroshiroSplitter,
        pos: &Vector3<i32>,
        density: f32,
        veins: Option<&VeinSample>,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        match self {
            Self::Aquifer(aquifer) => aquifer.apply(router, pos, density, height_estimator).0,
            Self::Ore(ore) => {
                veins.and_then(|veins| ore.sample(router, ore_random_deriver, pos, veins))
            }
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
        ore_random_deriver: &XoroshiroSplitter,
        pos: &Vector3<i32>,
        density: f32,
        veins: Option<&VeinSample>,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        for sampler in &mut self.samplers {
            if let Some(state) = sampler.sample(
                router,
                ore_random_deriver,
                pos,
                density,
                veins,
                height_estimator,
            ) {
                return Some(state);
            }
        }
        None
    }
}

pub struct ChunkNoiseGenerator<'a> {
    pub state_sampler: ChainedBlockStateSampler,
    generation_shape: &'a GenerationShapeConfig,
    volume: DensityVolume,
    ore_veins: bool,
    pub router: ChunkNoiseRouter<'a>,
}

impl<'a> ChunkNoiseGenerator<'a> {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        noise_router_base: &'a ProtoNoiseRouter,
        random_config: &GlobalRandomConfig,
        volume: DensityVolume,
        generation_shape: &'a GenerationShapeConfig,
        level_sampler: StandardChunkFluidLevelSampler,
        aquifers: bool,
        ore_veins: bool,
        beardifier_structures: Vec<
            crate::generation::noise::router::density_function::beardifier::BeardifierStructure,
        >,
        beardifier_junctions: Vec<
            crate::generation::noise::router::density_function::beardifier::BeardifierJunction,
        >,
        affected_box: Option<pumpkin_util::math::block_box::BlockBox>,
    ) -> Self {
        let builder_options = ChunkNoiseFunctionBuilderOptions::new(
            beardifier_structures,
            beardifier_junctions,
            affected_box,
        );

        let aquifer_sampler = if aquifers {
            let section_x = section_coords::block_to_section(volume.min_block_x);
            let section_z = section_coords::block_to_section(volume.min_block_z);
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
            volume,
            ore_veins,
            router,
        }
    }

    #[must_use]
    pub const fn volume(&self) -> &DensityVolume {
        &self.volume
    }

    pub fn sample_density(&mut self) -> ChunkDensities {
        let mut density = DensityBuffer::acquire(&self.volume);
        self.router.final_density_volume(&mut density, &self.volume);
        let veins = self.ore_veins.then(|| {
            let mut toggle = DensityBuffer::acquire(&self.volume);
            self.router.vein_toggle_volume(&mut toggle, &self.volume);
            let mut ridged = DensityBuffer::acquire(&self.volume);
            self.router.vein_ridged_volume(&mut ridged, &self.volume);
            [toggle, ridged]
        });
        ChunkDensities { density, veins }
    }

    pub fn sample_block_state(
        &mut self,
        ore_random_deriver: &XoroshiroSplitter,
        pos: &Vector3<i32>,
        density: f32,
        veins: Option<&VeinSample>,
        height_estimator: &mut SurfaceHeightEstimateSampler,
    ) -> Option<&'static BlockState> {
        self.state_sampler.sample(
            &mut self.router,
            ore_random_deriver,
            pos,
            density,
            veins,
            height_estimator,
        )
    }

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
