pub mod canyon;
pub mod cave;
pub mod mask;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use crate::generation::noise::CHUNK_DIM;
use crate::generation::noise::aquifer_sampler::AquiferSampler;
use crate::generation::noise::aquifer_sampler::AquiferSamplerImpl;
use crate::generation::noise::router::chunk_density_function::{
    ChunkNoiseFunctionBuilderOptions, ChunkNoiseFunctionSampleOptions, SampleAction,
};
use crate::generation::noise::router::chunk_noise_router::ChunkNoiseRouter;
use crate::generation::noise::router::surface_height_sampler::{
    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
};
use crate::generation::positions::chunk_pos::{start_block_x, start_block_z};
use pumpkin_data::BlockState;
use pumpkin_data::carver::{CANYON, CAVE, CAVE_EXTRA_UNDERGROUND, NETHER_CAVE};
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
use pumpkin_util::math::floor_div;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_carver_seed};

/// Partial port of vanilla `CarvingContext`. Holds only the noise pipeline
/// state the aquifer needs for `computeSubstance` at carver time.
///
/// Missing vs vanilla: `surfaceRule`, `randomState`, `noiseChunk`. Those are
/// what `CarvingContext.topMaterial(...)` runs against to place the biome's
/// surface block (grass/podzol/mycelium/etc.) on DIRT exposed below carved
/// ceilings. Until they're added, the `hasGrass` branch in `carve_block`
/// is a no-op.
pub struct CarvingContext<'g> {
    pub router: ChunkNoiseRouter<'g>,
    pub height_estimator: SurfaceHeightEstimateSampler<'g>,
    pub sample_options: ChunkNoiseFunctionSampleOptions,
}

impl<'g> CarvingContext<'g> {
    fn new(chunk_x: i32, chunk_z: i32, generator: &'g VanillaGenerator) -> Self {
        let generation_shape = &generator.settings.shape;
        let horizontal_cell_count = CHUNK_DIM / generation_shape.horizontal_cell_block_count();
        let start_x = start_block_x(chunk_x);
        let start_z = start_block_z(chunk_z);

        let vertical_cell_count = floor_div(
            generation_shape.height as usize,
            generation_shape.vertical_cell_block_count() as usize,
        );
        let horizontal_biome_end = crate::generation::biome_coords::from_block(
            horizontal_cell_count as i32 * generation_shape.horizontal_cell_block_count() as i32,
        );

        let builder_options = ChunkNoiseFunctionBuilderOptions::new(
            generation_shape.horizontal_cell_block_count() as usize,
            generation_shape.vertical_cell_block_count() as usize,
            vertical_cell_count,
            horizontal_cell_count as usize,
            crate::generation::biome_coords::from_block(start_x),
            crate::generation::biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
        );
        let router = ChunkNoiseRouter::generate(&generator.base_router.noise, &builder_options);

        let surface_config = SurfaceHeightSamplerBuilderOptions::new(
            crate::generation::biome_coords::from_block(start_x),
            crate::generation::biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
            generation_shape.min_y as i32,
            generation_shape.max_y() as i32,
            generation_shape.vertical_cell_block_count() as usize,
        );
        let height_estimator = SurfaceHeightEstimateSampler::generate(
            &generator.base_router.surface_estimator,
            &surface_config,
        );

        // Carvers do per-block random-access lookups, never cell-aligned fills,
        // so the cell caches don't help and we explicitly skip them.
        let sample_options =
            ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0);

        Self {
            router,
            height_estimator,
            sample_options,
        }
    }
}

pub trait Carver {
    #[allow(clippy::too_many_arguments)]
    fn carve(
        &self,
        config: &CarverConfig,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        legacy_random_source: bool,
        ctx: &mut CarvingContext,
        aquifer: &mut AquiferSampler,
    );
}

pub fn carve(chunk: &mut ProtoChunk, generator: &VanillaGenerator) {
    // Vanilla applyCarvers uses a range of 8 chunks (17x17 area)
    let radius = 8;
    let chunk_x = chunk.x;
    let chunk_z = chunk.z;
    let chunk_pos = Vector2::new(chunk_x, chunk_z);

    let overworld_carvers = [&CAVE, &CAVE_EXTRA_UNDERGROUND, &CANYON];
    let nether_carvers = [&NETHER_CAVE];

    let carvers_to_use = if generator.dimension == pumpkin_data::dimension::Dimension::OVERWORLD {
        &overworld_carvers[..]
    } else if generator.dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
        &nether_carvers[..]
    } else {
        &[]
    };

    let cave_carver = cave::CaveCarver;
    let canyon_carver = canyon::CanyonCarver;

    // Move the aquifer out of the chunk so `&mut chunk` and the aquifer can
    // be borrowed separately. Carvers are the last stage that needs it, so
    // it's intentionally consumed here. Mirrors vanilla pulling the aquifer
    // off the cached NoiseChunk via `noiseChunk.aquifer()`.
    let Some(mut aquifer) = chunk.aquifer.take() else {
        return;
    };
    let mut ctx = CarvingContext::new(chunk_x, chunk_z, generator);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let carver_x = chunk_x + dx;
            let carver_z = chunk_z + dz;
            let carver_chunk_pos = Vector2::new(carver_x, carver_z);

            // In vanilla, carvers are per-biome. Here we use the hardcoded list but
            // maintain the random seed logic.
            for (index, &config) in carvers_to_use.iter().enumerate() {
                let seed = get_carver_seed(
                    generator.random_config.seed + index as u64,
                    carver_x,
                    carver_z,
                );
                let mut carver_random = if generator.settings.legacy_random_source {
                    RandomGenerator::Legacy(
                        pumpkin_util::random::legacy_rand::LegacyRand::from_seed(seed),
                    )
                } else {
                    RandomGenerator::Xoroshiro(
                        pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(seed),
                    )
                };

                if should_carve(config, &mut carver_random) {
                    match config.additional {
                        CarverAdditionalConfig::Cave(_) | CarverAdditionalConfig::NetherCave(_) => {
                            cave_carver.carve(
                                config,
                                chunk,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                generator.settings.legacy_random_source,
                                &mut ctx,
                                &mut aquifer,
                            );
                        }
                        CarverAdditionalConfig::Canyon(_) => {
                            canyon_carver.carve(
                                config,
                                chunk,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                generator.settings.legacy_random_source,
                                &mut ctx,
                                &mut aquifer,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn should_carve(config: &CarverConfig, random: &mut RandomGenerator) -> bool {
    random.next_f32() <= config.probability
}

/// Vanilla `WorldCarver.getCarveState`: force LAVA below the carver's
/// `lava_level`, otherwise consult the aquifer (which may itself return
/// `None` for the vanilla barrier outcome).
#[allow(clippy::too_many_arguments)]
pub fn get_carve_state(
    chunk: &ProtoChunk,
    config: &CarverConfig,
    x: i32,
    y: i32,
    z: i32,
    ctx: &mut CarvingContext,
    aquifer: &mut AquiferSampler,
) -> Option<&'static BlockState> {
    let lava_y = config
        .lava_level
        .get_y(chunk.bottom_y() as i16, chunk.height());
    if y <= lava_y {
        Some(pumpkin_data::Block::LAVA.default_state)
    } else {
        // Vanilla: aquifer.computeSubstance(new SinglePointContext(x, y, z), 0.0).
        // TODO: vanilla wraps this in `isDebugEnabled(config) ? debugBarrier/debugState : ...`
        // debug worlds aren't modeled in pumpkin, so we pass the aquifer result through unchanged.
        aquifer.compute_substance(
            &mut ctx.router,
            &pumpkin_util::math::vector3::Vector3::new(x, y, z),
            &ctx.sample_options,
            &mut ctx.height_estimator,
            0.0,
        )
    }
}
