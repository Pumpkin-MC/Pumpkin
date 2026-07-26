mod apply;
mod levels;

use std::sync::LazyLock;

use pumpkin_data::{
    Block, chunk_gen_settings::GenerationSettings, dimension::Dimension,
    noise_router::OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::vector3::Vector3;

use crate::generation::{
    GlobalRandomConfig, biome_coords,
    noise::{
        BlockStateSampler, ChunkNoiseGenerator, LAVA_BLOCK, WATER_BLOCK,
        router::{
            chunk_density_function::{ChunkNoiseFunctionSampleOptions, SampleAction},
            chunk_noise_router::ChunkNoiseRouter,
            proto_noise_router::ProtoNoiseRouters,
            surface_height_sampler::{
                SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
            },
        },
    },
    positions::chunk_pos,
    proto_chunk::StandardChunkFluidLevelSampler,
};

use super::{
    AquiferSampler, CarverAquiferResult, CarverAquiferSampler, FluidLevel, SeaLevelAquiferSampler,
    WorldAquiferSampler,
};

const SEED: u64 = 0;
static RANDOM_CONFIG: LazyLock<GlobalRandomConfig> =
    LazyLock::new(|| GlobalRandomConfig::new(SEED, false));
static PROTO_ROUTER: LazyLock<ProtoNoiseRouters> = LazyLock::new(|| {
    let router_ast = &OVERWORLD_BASE_NOISE_ROUTER;
    ProtoNoiseRouters::generate(router_ast, &RANDOM_CONFIG)
});

#[expect(clippy::unreachable)]
fn create_aquifer(
    base_router: &'_ ProtoNoiseRouters,
) -> (
    WorldAquiferSampler,
    ChunkNoiseRouter<'_>,
    SurfaceHeightEstimateSampler<'_>,
    ChunkNoiseFunctionSampleOptions,
) {
    const CHUNK_WIDTH: usize = 16;

    let surface_config = GenerationSettings::from_dimension(&Dimension::OVERWORLD);
    let shape = &surface_config.shape;
    let chunk_x = 7;
    let chunk_z = 4;

    let sampler = StandardChunkFluidLevelSampler::new(
        FluidLevel::new(63, &WATER_BLOCK),
        FluidLevel::new(-54, &LAVA_BLOCK),
    );
    let noise = ChunkNoiseGenerator::new(
        &base_router.noise,
        &RANDOM_CONFIG,
        CHUNK_WIDTH / shape.horizontal_cell_block_count() as usize,
        chunk_pos::start_block_x(chunk_x),
        chunk_pos::start_block_z(chunk_z),
        shape,
        sampler,
        true,
        true,
        Vec::new(),
        Vec::new(),
        None,
    );
    let options =
        ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0);
    let mut samplers_vec = noise.state_sampler.samplers.into_vec();
    let first_sampler = samplers_vec.remove(0);

    let BlockStateSampler::Aquifer(sampler) = first_sampler else {
        panic!("Expected Aquifer")
    };

    let AquiferSampler::Aquifer(aquifer) = sampler else {
        unreachable!()
    };

    let horizontal_cell_count = CHUNK_WIDTH / shape.horizontal_cell_block_count() as usize;

    let horizontal_biome_end = biome_coords::from_block(
        horizontal_cell_count as i32 * shape.horizontal_cell_block_count() as i32,
    );

    let surface_height_estimator_options = SurfaceHeightSamplerBuilderOptions::new(
        chunk_x,
        chunk_z,
        horizontal_biome_end as usize,
        shape.min_y as i32,
        shape.max_y() as i32,
        shape.vertical_cell_block_count() as usize,
    );
    let height_estimator = SurfaceHeightEstimateSampler::generate(
        &base_router.surface_estimator,
        &surface_height_estimator_options,
    );

    (aquifer, noise.router, height_estimator, options)
}

fn create_carver_aquifer() -> CarverAquiferSampler<'static> {
    let settings = GenerationSettings::from_dimension(&Dimension::OVERWORLD);
    CarverAquiferSampler::new(7, 4, &PROTO_ROUTER, &RANDOM_CONFIG, settings)
}

#[test]
fn carver_aquifer_returns_stable_output() {
    let pos = Vector3::new(112, 0, 64);
    let mut first = create_carver_aquifer();
    let mut second = create_carver_aquifer();

    assert_eq!(first.compute(&pos, -1.0), second.compute(&pos, -1.0));
}

#[test]
fn carver_aquifer_handles_chunk_edges() {
    let mut aquifer = create_carver_aquifer();
    let positions = [
        Vector3::new(112, -64, 64),
        Vector3::new(127, -64, 79),
        Vector3::new(112, 319, 79),
        Vector3::new(127, 319, 64),
    ];

    for pos in positions {
        let _ = aquifer.compute(&pos, -1.0);
    }
}

#[test]
fn carver_aquifer_reports_fluid_schedule_signal() {
    let mut aquifer = create_carver_aquifer();
    let mut found_schedule = false;

    'positions: for y in -64..=63 {
        for x in 112..=127 {
            for z in 64..=79 {
                if aquifer
                    .compute(&Vector3::new(x, y, z), -1.0)
                    .should_schedule_fluid_update
                {
                    found_schedule = true;
                    break 'positions;
                }
            }
        }
    }

    assert!(found_schedule);
}

#[test]
#[expect(clippy::too_many_lines, clippy::large_stack_arrays)]
fn get_fluid_block_state() {
    let (_, mut router, _, options) = create_aquifer(&PROTO_ROUTER);
    let level = FluidLevel::new(0, &WATER_BLOCK);

    let values = [
        ((-100, -100, -100), WATER_BLOCK),
        ((-100, -100, -50), LAVA_BLOCK),
        ((-100, -100, 0), WATER_BLOCK),
        ((-100, -100, 50), WATER_BLOCK),
        ((-100, -100, 100), WATER_BLOCK),
        ((-100, -50, -100), WATER_BLOCK),
        ((-100, -50, -50), LAVA_BLOCK),
        ((-100, -50, 0), LAVA_BLOCK),
        ((-100, -50, 50), LAVA_BLOCK),
        ((-100, -50, 100), WATER_BLOCK),
        ((-100, 0, -100), WATER_BLOCK),
        ((-100, 0, -50), WATER_BLOCK),
        ((-100, 0, 0), WATER_BLOCK),
        ((-100, 0, 50), WATER_BLOCK),
        ((-100, 0, 100), WATER_BLOCK),
        ((-100, 50, -100), WATER_BLOCK),
        ((-100, 50, -50), WATER_BLOCK),
        ((-100, 50, 0), WATER_BLOCK),
        ((-100, 50, 50), WATER_BLOCK),
        ((-100, 50, 100), WATER_BLOCK),
        ((-100, 100, -100), WATER_BLOCK),
        ((-100, 100, -50), WATER_BLOCK),
        ((-100, 100, 0), WATER_BLOCK),
        ((-100, 100, 50), WATER_BLOCK),
        ((-100, 100, 100), WATER_BLOCK),
        ((-50, -100, -100), WATER_BLOCK),
        ((-50, -100, -50), WATER_BLOCK),
        ((-50, -100, 0), LAVA_BLOCK),
        ((-50, -100, 50), LAVA_BLOCK),
        ((-50, -100, 100), WATER_BLOCK),
        ((-50, -50, -100), WATER_BLOCK),
        ((-50, -50, -50), WATER_BLOCK),
        ((-50, -50, 0), WATER_BLOCK),
        ((-50, -50, 50), WATER_BLOCK),
        ((-50, -50, 100), WATER_BLOCK),
        ((-50, 0, -100), LAVA_BLOCK),
        ((-50, 0, -50), WATER_BLOCK),
        ((-50, 0, 0), WATER_BLOCK),
        ((-50, 0, 50), WATER_BLOCK),
        ((-50, 0, 100), WATER_BLOCK),
        ((-50, 50, -100), WATER_BLOCK),
        ((-50, 50, -50), WATER_BLOCK),
        ((-50, 50, 0), LAVA_BLOCK),
        ((-50, 50, 50), LAVA_BLOCK),
        ((-50, 50, 100), WATER_BLOCK),
        ((-50, 100, -100), WATER_BLOCK),
        ((-50, 100, -50), WATER_BLOCK),
        ((-50, 100, 0), LAVA_BLOCK),
        ((-50, 100, 50), LAVA_BLOCK),
        ((-50, 100, 100), LAVA_BLOCK),
        ((0, -100, -100), WATER_BLOCK),
        ((0, -100, -50), LAVA_BLOCK),
        ((0, -100, 0), LAVA_BLOCK),
        ((0, -100, 50), LAVA_BLOCK),
        ((0, -100, 100), WATER_BLOCK),
        ((0, -50, -100), WATER_BLOCK),
        ((0, -50, -50), WATER_BLOCK),
        ((0, -50, 0), WATER_BLOCK),
        ((0, -50, 50), WATER_BLOCK),
        ((0, -50, 100), WATER_BLOCK),
        ((0, 0, -100), LAVA_BLOCK),
        ((0, 0, -50), LAVA_BLOCK),
        ((0, 0, 0), WATER_BLOCK),
        ((0, 0, 50), WATER_BLOCK),
        ((0, 0, 100), WATER_BLOCK),
        ((0, 50, -100), WATER_BLOCK),
        ((0, 50, -50), WATER_BLOCK),
        ((0, 50, 0), WATER_BLOCK),
        ((0, 50, 50), WATER_BLOCK),
        ((0, 50, 100), WATER_BLOCK),
        ((0, 100, -100), WATER_BLOCK),
        ((0, 100, -50), LAVA_BLOCK),
        ((0, 100, 0), WATER_BLOCK),
        ((0, 100, 50), WATER_BLOCK),
        ((0, 100, 100), WATER_BLOCK),
        ((50, -100, -100), WATER_BLOCK),
        ((50, -100, -50), LAVA_BLOCK),
        ((50, -100, 0), LAVA_BLOCK),
        ((50, -100, 50), LAVA_BLOCK),
        ((50, -100, 100), WATER_BLOCK),
        ((50, -50, -100), WATER_BLOCK),
        ((50, -50, -50), WATER_BLOCK),
        ((50, -50, 0), WATER_BLOCK),
        ((50, -50, 50), WATER_BLOCK),
        ((50, -50, 100), WATER_BLOCK),
        ((50, 0, -100), LAVA_BLOCK),
        ((50, 0, -50), LAVA_BLOCK),
        ((50, 0, 0), WATER_BLOCK),
        ((50, 0, 50), WATER_BLOCK),
        ((50, 0, 100), WATER_BLOCK),
        ((50, 50, -100), WATER_BLOCK),
        ((50, 50, -50), WATER_BLOCK),
        ((50, 50, 0), WATER_BLOCK),
        ((50, 50, 50), WATER_BLOCK),
        ((50, 50, 100), WATER_BLOCK),
        ((50, 100, -100), WATER_BLOCK),
        ((50, 100, -50), LAVA_BLOCK),
        ((50, 100, 0), WATER_BLOCK),
        ((50, 100, 50), WATER_BLOCK),
        ((50, 100, 100), WATER_BLOCK),
        ((100, -100, -100), WATER_BLOCK),
        ((100, -100, -50), LAVA_BLOCK),
        ((100, -100, 0), WATER_BLOCK),
        ((100, -100, 50), WATER_BLOCK),
        ((100, -100, 100), WATER_BLOCK),
        ((100, -50, -100), LAVA_BLOCK),
        ((100, -50, -50), LAVA_BLOCK),
        ((100, -50, 0), LAVA_BLOCK),
        ((100, -50, 50), LAVA_BLOCK),
        ((100, -50, 100), LAVA_BLOCK),
        ((100, 0, -100), WATER_BLOCK),
        ((100, 0, -50), LAVA_BLOCK),
        ((100, 0, 0), WATER_BLOCK),
        ((100, 0, 50), WATER_BLOCK),
        ((100, 0, 100), LAVA_BLOCK),
        ((100, 50, -100), WATER_BLOCK),
        ((100, 50, -50), WATER_BLOCK),
        ((100, 50, 0), WATER_BLOCK),
        ((100, 50, 50), WATER_BLOCK),
        ((100, 50, 100), WATER_BLOCK),
        ((100, 100, -100), LAVA_BLOCK),
        ((100, 100, -50), LAVA_BLOCK),
        ((100, 100, 0), WATER_BLOCK),
        ((100, 100, 50), WATER_BLOCK),
        ((100, 100, 100), WATER_BLOCK),
    ];

    for ((x, y, z), result) in values {
        assert_eq!(
            WorldAquiferSampler::get_fluid_block_state(x, y, z, &level, -10, &mut router, &options),
            &result
        );
    }
}

#[test]
#[expect(clippy::too_many_lines)]
fn get_noise_based_fluid_level() {
    let (_, mut router, _, options) = create_aquifer(&PROTO_ROUTER);

    let values = [
        ((-100, -100, -100), -103),
        ((-100, -100, -50), -103),
        ((-100, -100, 0), -103),
        ((-100, -100, 50), -103),
        ((-100, -100, 100), -103),
        ((-100, -50, -100), -63),
        ((-100, -50, -50), -63),
        ((-100, -50, 0), -63),
        ((-100, -50, 50), -63),
        ((-100, -50, 100), -63),
        ((-100, 0, -100), 17),
        ((-100, 0, -50), 17),
        ((-100, 0, 0), 17),
        ((-100, 0, 50), 17),
        ((-100, 0, 100), 17),
        ((-100, 50, -100), 57),
        ((-100, 50, -50), 57),
        ((-100, 50, 0), 57),
        ((-100, 50, 50), 57),
        ((-100, 50, 100), 57),
        ((-100, 100, -100), 97),
        ((-100, 100, -50), 97),
        ((-100, 100, 0), 97),
        ((-100, 100, 50), 97),
        ((-100, 100, 100), 97),
        ((-50, -100, -100), -103),
        ((-50, -100, -50), -103),
        ((-50, -100, 0), -103),
        ((-50, -100, 50), -103),
        ((-50, -100, 100), -100),
        ((-50, -50, -100), -63),
        ((-50, -50, -50), -63),
        ((-50, -50, 0), -63),
        ((-50, -50, 50), -63),
        ((-50, -50, 100), -60),
        ((-50, 0, -100), 17),
        ((-50, 0, -50), 17),
        ((-50, 0, 0), 17),
        ((-50, 0, 50), 17),
        ((-50, 0, 100), 20),
        ((-50, 50, -100), 57),
        ((-50, 50, -50), 57),
        ((-50, 50, 0), 57),
        ((-50, 50, 50), 57),
        ((-50, 50, 100), 60),
        ((-50, 100, -100), 97),
        ((-50, 100, -50), 97),
        ((-50, 100, 0), 97),
        ((-50, 100, 50), 97),
        ((-50, 100, 100), 100),
        ((0, -100, -100), -103),
        ((0, -100, -50), -103),
        ((0, -100, 0), -103),
        ((0, -100, 50), -100),
        ((0, -100, 100), -100),
        ((0, -50, -100), -63),
        ((0, -50, -50), -63),
        ((0, -50, 0), -63),
        ((0, -50, 50), -60),
        ((0, -50, 100), -60),
        ((0, 0, -100), 17),
        ((0, 0, -50), 17),
        ((0, 0, 0), 17),
        ((0, 0, 50), 20),
        ((0, 0, 100), 20),
        ((0, 50, -100), 57),
        ((0, 50, -50), 57),
        ((0, 50, 0), 57),
        ((0, 50, 50), 60),
        ((0, 50, 100), 60),
        ((0, 100, -100), 97),
        ((0, 100, -50), 97),
        ((0, 100, 0), 97),
        ((0, 100, 50), 100),
        ((0, 100, 100), 100),
        ((50, -100, -100), -103),
        ((50, -100, -50), -103),
        ((50, -100, 0), -103),
        ((50, -100, 50), -100),
        ((50, -100, 100), -100),
        ((50, -50, -100), -63),
        ((50, -50, -50), -63),
        ((50, -50, 0), -63),
        ((50, -50, 50), -60),
        ((50, -50, 100), -60),
        ((50, 0, -100), 17),
        ((50, 0, -50), 17),
        ((50, 0, 0), 17),
        ((50, 0, 50), 20),
        ((50, 0, 100), 20),
        ((50, 50, -100), 57),
        ((50, 50, -50), 57),
        ((50, 50, 0), 57),
        ((50, 50, 50), 60),
        ((50, 50, 100), 60),
        ((50, 100, -100), 97),
        ((50, 100, -50), 97),
        ((50, 100, 0), 97),
        ((50, 100, 50), 100),
        ((50, 100, 100), 100),
        ((100, -100, -100), -103),
        ((100, -100, -50), -103),
        ((100, -100, 0), -103),
        ((100, -100, 50), -103),
        ((100, -100, 100), -100),
        ((100, -50, -100), -63),
        ((100, -50, -50), -63),
        ((100, -50, 0), -63),
        ((100, -50, 50), -63),
        ((100, -50, 100), -60),
        ((100, 0, -100), 17),
        ((100, 0, -50), 17),
        ((100, 0, 0), 17),
        ((100, 0, 50), 17),
        ((100, 0, 100), 20),
        ((100, 50, -100), 57),
        ((100, 50, -50), 57),
        ((100, 50, 0), 57),
        ((100, 50, 50), 57),
        ((100, 50, 100), 60),
        ((100, 100, -100), 97),
        ((100, 100, -50), 97),
        ((100, 100, 0), 97),
        ((100, 100, 50), 97),
        ((100, 100, 100), 100),
    ];

    for ((x, y, z), result) in values {
        assert_eq!(
            WorldAquiferSampler::get_noise_based_fluid_level(x, y, z, 200, &mut router, &options),
            result
        );
    }
}

#[test]
#[expect(clippy::too_many_lines)]
fn calculate_density() {
    let (_, mut router, _, env) = create_aquifer(&PROTO_ROUTER);

    let values = [
        ((-100, -100, -100, 0, 0), 0.0),
        ((-100, -100, -50, 50, 0), -19.3),
        ((-100, -100, 0, 0, 0), 0.0),
        ((-100, -100, 50, 50, 0), -19.3),
        ((-100, -100, 100, 0, 0), 0.0),
        ((-100, -50, -100, 0, 0), 0.0),
        ((-100, -50, -50, 50, 0), -9.3),
        ((-100, -50, 0, 0, 0), 0.0),
        ((-100, -50, 50, 50, 0), -9.3),
        ((-100, -50, 100, 0, 0), 0.0),
        ((-100, 0, -100, 0, 0), 0.0),
        ((-100, 0, -50, -50, 0), 0.2083850667904572),
        ((-100, 0, 0, 0, 0), 0.0),
        ((-100, 0, 50, 50, 0), 2.069189235272414),
        ((-100, 0, 100, 0, 0), 0.0),
        ((-100, 50, -100, 0, 0), 0.0),
        ((-100, 50, -50, -50, 0), -40.4),
        ((-100, 50, 0, 0, 0), 0.0),
        ((-100, 50, 50, -50, 0), -40.4),
        ((-100, 50, 100, 0, 0), 0.0),
        ((-100, 100, -100, 0, 0), 0.0),
        ((-100, 100, -50, -50, 0), -80.4),
        ((-100, 100, 0, 0, 0), 0.0),
        ((-100, 100, 50, -50, 0), -80.4),
        ((-100, 100, 100, 0, 0), 0.0),
        ((-50, -100, -100, 0, 50), -19.3),
        ((-50, -100, -50, 50, 50), 0.0),
        ((-50, -100, 0, 0, -50), -9.3),
        ((-50, -100, 50, 50, -50), -9.3),
        ((-50, -100, 100, 0, -50), -9.3),
        ((-50, -50, -100, 0, 50), -9.3),
        ((-50, -50, -50, 50, 50), 0.0),
        ((-50, -50, 0, 0, -50), 2.2042949518442185),
        ((-50, -50, 50, 50, -50), 1.8767275908406176),
        ((-50, -50, 100, 0, -50), 2.3399656359995133),
        ((-50, 0, -100, 0, -50), -0.08405949841069171),
        ((-50, 0, -50, -50, -50), 0.0),
        ((-50, 0, 0, 0, -50), 0.3902410585192353),
        ((-50, 0, 50, 50, -50), 66.0),
        ((-50, 0, 100, 0, -50), -0.7930165090675787),
        ((-50, 50, -100, 0, -50), -40.4),
        ((-50, 50, -50, -50, -50), 0.0),
        ((-50, 50, 0, 0, -50), -40.4),
        ((-50, 50, 50, -50, 50), -0.35570822400215646),
        ((-50, 50, 100, 0, 50), -0.16770224497207317),
        ((-50, 100, -100, 0, -50), -80.4),
        ((-50, 100, -50, -50, -50), 0.0),
        ((-50, 100, 0, 0, -50), -80.4),
        ((-50, 100, 50, -50, 50), -40.4),
        ((-50, 100, 100, 0, 50), -40.4),
        ((0, -100, -100, 0, 0), 0.0),
        ((0, -100, -50, -50, 0), -9.3),
        ((0, -100, 0, 0, 0), 0.0),
        ((0, -100, 50, 50, 0), -19.3),
        ((0, -100, 100, 0, 0), 0.0),
        ((0, -50, -100, 0, 0), 0.0),
        ((0, -50, -50, -50, 0), 2.857141340264507),
        ((0, -50, 0, 0, 0), 0.0),
        ((0, -50, 50, 50, 0), -9.3),
        ((0, -50, 100, 0, 0), 0.0),
        ((0, 0, -100, 0, 0), 0.0),
        ((0, 0, -50, -50, 0), -0.1361016501707068),
        ((0, 0, 0, 0, 0), 0.0),
        ((0, 0, 50, 50, 0), 1.9841279541408636),
        ((0, 0, 100, 0, 0), 0.0),
        ((0, 50, -100, 0, 0), 0.0),
        ((0, 50, -50, -50, 0), -40.4),
        ((0, 50, 0, 0, 0), 0.0),
        ((0, 50, 50, 50, 0), -0.36331007530382964),
        ((0, 50, 100, 0, 0), 0.0),
        ((0, 100, -100, 0, 0), 0.0),
        ((0, 100, -50, -50, 0), -80.4),
        ((0, 100, 0, 0, 0), 0.0),
        ((0, 100, 50, 50, 0), -40.4),
        ((0, 100, 100, 0, 0), 0.0),
        ((50, -100, -100, 0, 50), -19.3),
        ((50, -100, -50, -50, 50), -9.3),
        ((50, -100, 0, 0, 50), -19.3),
        ((50, -100, 50, -50, -50), 0.0),
        ((50, -100, 100, 0, -50), -9.3),
        ((50, -50, -100, 0, 50), -9.3),
        ((50, -50, -50, -50, 50), 1.619242225388449),
        ((50, -50, 0, 0, 50), -9.3),
        ((50, -50, 50, -50, -50), 0.0),
        ((50, -50, 100, 0, -50), 2.1561171703198188),
        ((50, 0, -100, 0, 50), 2.6298865590685954),
        ((50, 0, -50, -50, 50), 66.0),
        ((50, 0, 0, 0, 50), 2.572198917833846),
        ((50, 0, 50, 50, 50), 0.0),
        ((50, 0, 100, 0, 50), 2.082884998883258),
        ((50, 50, -100, 0, -50), -40.4),
        ((50, 50, -50, 50, -50), -0.1894344852785401),
        ((50, 50, 0, 0, 50), -0.7155260733519367),
        ((50, 50, 50, 50, 50), 0.0),
        ((50, 50, 100, 0, 50), -0.4132183490530098),
        ((50, 100, -100, 0, -50), -80.4),
        ((50, 100, -50, 50, -50), -40.4),
        ((50, 100, 0, 0, 50), -40.4),
        ((50, 100, 50, 50, 50), 0.0),
        ((50, 100, 100, 0, 50), -40.4),
        ((100, -100, -100, 0, 0), 0.0),
        ((100, -100, -50, -50, 0), -9.3),
        ((100, -100, 0, 0, 0), 0.0),
        ((100, -100, 50, -50, 0), -9.3),
        ((100, -100, 100, 0, 0), 0.0),
        ((100, -50, -100, 0, 0), 0.0),
        ((100, -50, -50, -50, 0), 1.6711026207576742),
        ((100, -50, 0, 0, 0), 0.0),
        ((100, -50, 50, -50, 0), 2.042353012197518),
        ((100, -50, 100, 0, 0), 0.0),
        ((100, 0, -100, 0, 0), 0.0),
        ((100, 0, -50, -50, 0), 0.3145492757856567),
        ((100, 0, 0, 0, 0), 0.0),
        ((100, 0, 50, 50, 0), 2.27260703684609),
        ((100, 0, 100, 0, 0), 0.0),
        ((100, 50, -100, 0, 0), 0.0),
        ((100, 50, -50, 50, 0), -0.16949328993376553),
        ((100, 50, 0, 0, 0), 0.0),
        ((100, 50, 50, 50, 0), 0.5196380801381327),
        ((100, 50, 100, 0, 0), 0.0),
        ((100, 100, -100, 0, 0), 0.0),
        ((100, 100, -50, 50, 0), -40.4),
        ((100, 100, 0, 0, 0), 0.0),
        ((100, 100, 50, 50, 0), -40.4),
        ((100, 100, 100, 0, 0), 0.0),
    ];

    for ((x, y, z, h1, h2), result) in values {
        let level1 = FluidLevel::new(h1, &WATER_BLOCK);
        let level2 = FluidLevel::new(h2, &WATER_BLOCK);
        let pos = Vector3::new(x, y, z);
        let mut sample = None;

        assert_eq!(
            WorldAquiferSampler::calculate_density(
                &mut sample,
                &pos,
                &mut router,
                &env,
                &level1,
                &level2
            ),
            result
        );
    }
}

#[test]
fn max_distance_is_linear_in_similarity() {
    assert_eq!(WorldAquiferSampler::max_distance(0, 0), 1.0);
    assert_eq!(WorldAquiferSampler::max_distance(0, 25), 0.0);
    assert_eq!(
        WorldAquiferSampler::max_distance(-10, 10),
        WorldAquiferSampler::max_distance(10, -10)
    );
}

#[test]
fn fluid_level_reports_block_below_surface_only() {
    let level = FluidLevel::new(10, &WATER_BLOCK);
    assert_eq!(level.max_y_exclusive(), 10);
    assert_eq!(level.get_block(9), &WATER_BLOCK);
    assert_eq!(level.get_block(10), &Block::AIR);
}

#[test]
fn moved_sampler_apis_remain_reachable() {
    let sampler = StandardChunkFluidLevelSampler::new(
        FluidLevel::new(63, &WATER_BLOCK),
        FluidLevel::new(-54, &LAVA_BLOCK),
    );
    let _sea_level = SeaLevelAquiferSampler::new(sampler);
    let _: fn(&mut CarverAquiferSampler<'_>, &Vector3<i32>, f64) -> CarverAquiferResult =
        CarverAquiferSampler::compute;
}
