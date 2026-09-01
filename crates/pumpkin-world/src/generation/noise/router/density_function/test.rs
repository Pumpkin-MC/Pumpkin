#![allow(
    clippy::excessive_precision,
    clippy::redundant_test_prefix,
    clippy::items_after_statements
)]

use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
use pumpkin_util::math::vector3::Vector3;
use std::sync::LazyLock;

use crate::generation::GlobalRandomConfig;
use crate::generation::noise::router::chunk_density_function::ChunkNoiseFunctionBuilderOptions;
use crate::generation::noise::router::chunk_density_function::{
    ChunkNoiseFunctionSampleOptions, SampleAction,
};
use crate::generation::noise::router::chunk_noise_router::{
    ChunkNoiseDensityFunction, ChunkNoiseFunctionComponent,
};
use crate::generation::noise::router::proto_noise_router::{
    ProtoNoiseFunctionComponent, ProtoNoiseRouters,
};

use super::{NoiseFunctionComponentRange, PassThrough};

// This is a dummy value because we are not actually building chunk-specific functions
static TEST_OPTIONS: ChunkNoiseFunctionBuilderOptions =
    ChunkNoiseFunctionBuilderOptions::new(0, 0, 0, 0, 0, 0, 0, Vec::new(), Vec::new(), None);
const SEED: u64 = 0;
static RANDOM_CONFIG: LazyLock<GlobalRandomConfig> =
    LazyLock::new(|| GlobalRandomConfig::new(SEED, false));

macro_rules! build_function_stack {
    ($stack:expr) => {{
        $stack
            .iter()
            .map(|component| match component {
                ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                    ChunkNoiseFunctionComponent::PassThrough(PassThrough::new(
                        wrapper.input_index,
                        wrapper.min(),
                        wrapper.max(),
                    ))
                }
                ProtoNoiseFunctionComponent::PassThrough(pass_through) => {
                    ChunkNoiseFunctionComponent::PassThrough(pass_through.clone())
                }
                ProtoNoiseFunctionComponent::Dependent(dependent) => {
                    ChunkNoiseFunctionComponent::Dependent(&dependent)
                }
                ProtoNoiseFunctionComponent::Independent(independent) => {
                    ChunkNoiseFunctionComponent::Independent(&independent)
                }
                ProtoNoiseFunctionComponent::Beardifier(beardifier) => {
                    ChunkNoiseFunctionComponent::Chunk(
                        crate::generation::noise::router::chunk_density_function::ChunkSpecificNoiseFunctionComponent::Beardifier(
                            beardifier.clone(),
                        ),
                    )
                }
            })
            .collect::<Vec<_>>()
    }};
}

macro_rules! build_function {
    ($stack:expr) => {
        ChunkNoiseDensityFunction {
            component_stack: &mut $stack,
        }
    };
}

const TEST_SAMPLE_OPTIONS: ChunkNoiseFunctionSampleOptions =
    ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0);

macro_rules! sample_noise_router_function {
    ($name:ident, $pos: expr) => {{
        let base_router = &OVERWORLD_BASE_NOISE_ROUTER.noise;
        let proto_stack = ProtoNoiseRouters::generate_proto_stack(
            &base_router.full_component_stack,
            &RANDOM_CONFIG,
        );
        let mut stack = build_function_stack!(&proto_stack[..=base_router.$name]);
        let mut function = build_function!(&mut stack);
        function.sample(&$pos, &TEST_SAMPLE_OPTIONS)
    }};
}

macro_rules! sample_multi_noise_router_function {
    ($name:ident, $pos: expr) => {{
        let base_router = &OVERWORLD_BASE_NOISE_ROUTER.multi_noise;
        let proto_stack = ProtoNoiseRouters::generate_proto_stack(
            &base_router.full_component_stack,
            &RANDOM_CONFIG,
        );
        let mut stack = build_function_stack!(&proto_stack[..=base_router.$name]);
        let mut function = build_function!(&mut stack);
        function.sample(&$pos, &TEST_SAMPLE_OPTIONS)
    }};
}

macro_rules! sample_surface_router_function {
    ($pos: expr) => {{
        let base_router = &OVERWORLD_BASE_NOISE_ROUTER.surface_estimator;
        let proto_stack = ProtoNoiseRouters::generate_proto_stack(
            &base_router.full_component_stack,
            &RANDOM_CONFIG,
        );
        let mut stack = build_function_stack!(&proto_stack);
        let mut function = build_function!(&mut stack);
        function.sample(&$pos, &TEST_SAMPLE_OPTIONS)
    }};
}

// TODO: Test all dimensions/noise routers

#[test]
// This test verifies that the generated functions after seed initialization but before chunk
// initialization matches the respected Java values.
//
// This is equivalent to a Java `NoiseRouter` after being passed into `NoiseConfig` but before being
// passed into `ChunkNoiseGenerator`
#[expect(clippy::too_many_lines)]
fn normal_surface_noisified() {
    let pos = Vector3 { x: 0, y: 0, z: 0 };
    // TODO: Move these values to a file and create an extractor for them
    assert_eq!(
        sample_noise_router_function!(barrier_noise, pos),
        -0.54002273f32
    );
    assert_eq!(
        sample_noise_router_function!(fluid_level_floodedness_noise, pos),
        -0.4709572f32
    );
    assert_eq!(
        sample_noise_router_function!(fluid_level_spread_noise, pos),
        -0.05726914f32
    );
    assert_eq!(
        sample_noise_router_function!(lava_noise, pos),
        -0.16423604f32
    );
    assert_eq!(
        sample_multi_noise_router_function!(temperature, pos),
        0.11823799f32
    );
    assert_eq!(
        sample_multi_noise_router_function!(vegetation, pos),
        -0.0013601681f32
    );
    assert_eq!(
        sample_multi_noise_router_function!(continents, pos),
        -0.008171953f32
    );
    assert_eq!(
        sample_multi_noise_router_function!(erosion, pos),
        -0.10391074f32
    );
    assert_eq!(
        sample_multi_noise_router_function!(depth, pos),
        0.4118821f32
    );
    assert_eq!(
        sample_multi_noise_router_function!(ridges, pos),
        0.011110324f32
    );
    assert_eq!(sample_surface_router_function!(pos), 45.832787f32);
    assert_eq!(
        sample_noise_router_function!(final_density, pos),
        0.15719144f32
    );

    let values = [
        ((-100, -200, -100), 0.0f32),
        ((-100, -200, -50), 0.0f32),
        ((-100, -200, 0), 0.0f32),
        ((-100, -200, 50), 0.0f32),
        ((-100, -200, 100), 0.0f32),
        ((-100, -100, -100), 0.0f32),
        ((-100, -100, -50), 0.0f32),
        ((-100, -100, 0), 0.0f32),
        ((-100, -100, 50), 0.0f32),
        ((-100, -100, 100), 0.0f32),
        ((-100, 0, -100), 0.3462291472930333f32),
        ((-100, 0, -50), 0.2340445906392791f32),
        ((-100, 0, 0), -0.028825983399710407f32),
        ((-100, 0, 50), -0.16684760357850822f32),
        ((-100, 0, 100), -0.1843465939249143f32),
        ((-100, 100, -100), 0.0f32),
        ((-100, 100, -50), 0.0f32),
        ((-100, 100, 0), 0.0f32),
        ((-100, 100, 50), 0.0f32),
        ((-100, 100, 100), 0.0f32),
        ((-100, 200, -100), 0.0f32),
        ((-100, 200, -50), 0.0f32),
        ((-100, 200, 0), 0.0f32),
        ((-100, 200, 50), 0.0f32),
        ((-100, 200, 100), 0.0f32),
        ((-50, -200, -100), 0.0f32),
        ((-50, -200, -50), 0.0f32),
        ((-50, -200, 0), 0.0f32),
        ((-50, -200, 50), 0.0f32),
        ((-50, -200, 100), 0.0f32),
        ((-50, -100, -100), 0.0f32),
        ((-50, -100, -50), 0.0f32),
        ((-50, -100, 0), 0.0f32),
        ((-50, -100, 50), 0.0f32),
        ((-50, -100, 100), 0.0f32),
        ((-50, 0, -100), 0.05757810206373369f32),
        ((-50, 0, -50), 0.0014520730707135465f32),
        ((-50, 0, 0), -0.024149735708339466f32),
        ((-50, 0, 50), 0.1287619466526521f32),
        ((-50, 0, 100), 0.25507593901831094f32),
        ((-50, 100, -100), 0.0f32),
        ((-50, 100, -50), 0.0f32),
        ((-50, 100, 0), 0.0f32),
        ((-50, 100, 50), 0.0f32),
        ((-50, 100, 100), 0.0f32),
        ((-50, 200, -100), 0.0f32),
        ((-50, 200, -50), 0.0f32),
        ((-50, 200, 0), 0.0f32),
        ((-50, 200, 50), 0.0f32),
        ((-50, 200, 100), 0.0f32),
        ((0, -200, -100), 0.0f32),
        ((0, -200, -50), 0.0f32),
        ((0, -200, 0), 0.0f32),
        ((0, -200, 50), 0.0f32),
        ((0, -200, 100), 0.0f32),
        ((0, -100, -100), 0.0f32),
        ((0, -100, -50), 0.0f32),
        ((0, -100, 0), 0.0f32),
        ((0, -100, 50), 0.0f32),
        ((0, -100, 100), 0.0f32),
        ((0, 0, -100), -0.24030906682975775f32),
        ((0, 0, -50), -0.24705110006127165f32),
        ((0, 0, 0), -0.06643453056181631f32),
        ((0, 0, 50), 0.25318680526509063f32),
        ((0, 0, 100), 0.48257536249146743f32),
        ((0, 100, -100), 0.0f32),
        ((0, 100, -50), 0.0f32),
        ((0, 100, 0), 0.0f32),
        ((0, 100, 50), 0.0f32),
        ((0, 100, 100), 0.0f32),
        ((0, 200, -100), 0.0f32),
        ((0, 200, -50), 0.0f32),
        ((0, 200, 0), 0.0f32),
        ((0, 200, 50), 0.0f32),
        ((0, 200, 100), 0.0f32),
        ((50, -200, -100), 0.0f32),
        ((50, -200, -50), 0.0f32),
        ((50, -200, 0), 0.0f32),
        ((50, -200, 50), 0.0f32),
        ((50, -200, 100), 0.0f32),
        ((50, -100, -100), 0.0f32),
        ((50, -100, -50), 0.0f32),
        ((50, -100, 0), 0.0f32),
        ((50, -100, 50), 0.0f32),
        ((50, -100, 100), 0.0f32),
        ((50, 0, -100), 0.035583298926324954f32),
        ((50, 0, -50), -0.07225351839505538f32),
        ((50, 0, 0), -0.03474107481998612f32),
        ((50, 0, 50), 0.12616421777330467f32),
        ((50, 0, 100), 0.35414843965758613f32),
        ((50, 100, -100), 0.0f32),
        ((50, 100, -50), 0.0f32),
        ((50, 100, 0), 0.0f32),
        ((50, 100, 50), 0.0f32),
        ((50, 100, 100), 0.0f32),
        ((50, 200, -100), 0.0f32),
        ((50, 200, -50), 0.0f32),
        ((50, 200, 0), 0.0f32),
        ((50, 200, 50), 0.0f32),
        ((50, 200, 100), 0.0f32),
        ((100, -200, -100), 0.0f32),
        ((100, -200, -50), 0.0f32),
        ((100, -200, 0), 0.0f32),
        ((100, -200, 50), 0.0f32),
        ((100, -200, 100), 0.0f32),
        ((100, -100, -100), 0.0f32),
        ((100, -100, -50), 0.0f32),
        ((100, -100, 0), 0.0f32),
        ((100, -100, 50), 0.0f32),
        ((100, -100, 100), 0.0f32),
        ((100, 0, -100), 0.4151489417623382f32),
        ((100, 0, -50), 0.2092632456905039f32),
        ((100, 0, 0), -0.009920164828456044f32),
        ((100, 0, 50), -0.14997295538707048f32),
        ((100, 0, 100), -0.05777616780034325f32),
        ((100, 100, -100), 0.0f32),
        ((100, 100, -50), 0.0f32),
        ((100, 100, 0), 0.0f32),
        ((100, 100, 50), 0.0f32),
        ((100, 100, 100), 0.0f32),
        ((100, 200, -100), 0.0f32),
        ((100, 200, -50), 0.0f32),
        ((100, 200, 0), 0.0f32),
        ((100, 200, 50), 0.0f32),
        ((100, 200, 100), 0.0f32),
    ];
    for ((x, y, z), value) in &values {
        let pos = Vector3 {
            x: *x,
            y: *y,
            z: *z,
        };
        assert_eq!(sample_noise_router_function!(vein_toggle, pos), *value);
    }

    let values = [
        ((-100, -200, -100), -1.0f32),
        ((-100, -200, -50), -1.0f32),
        ((-100, -200, 0), -1.0f32),
        ((-100, -200, 50), -1.0f32),
        ((-100, -200, 100), -1.0f32),
        ((-100, -100, -100), -1.0f32),
        ((-100, -100, -50), -1.0f32),
        ((-100, -100, 0), -1.0f32),
        ((-100, -100, 50), -1.0f32),
        ((-100, -100, 100), -1.0f32),
        ((-100, 0, -100), -1.0f32),
        ((-100, 0, -50), -1.0f32),
        ((-100, 0, 0), -1.0f32),
        ((-100, 0, 50), -1.0f32),
        ((-100, 0, 100), -1.0f32),
        ((-100, 100, -100), -1.0f32),
        ((-100, 100, -50), -1.0f32),
        ((-100, 100, 0), -1.0f32),
        ((-100, 100, 50), -1.0f32),
        ((-100, 100, 100), -1.0f32),
        ((-100, 200, -100), -1.0f32),
        ((-100, 200, -50), -1.0f32),
        ((-100, 200, 0), -1.0f32),
        ((-100, 200, 50), -1.0f32),
        ((-100, 200, 100), -1.0f32),
        ((-50, -200, -100), -1.0f32),
        ((-50, -200, -50), -1.0f32),
        ((-50, -200, 0), -1.0f32),
        ((-50, -200, 50), -1.0f32),
        ((-50, -200, 100), -1.0f32),
        ((-50, -100, -100), -1.0f32),
        ((-50, -100, -50), -1.0f32),
        ((-50, -100, 0), -1.0f32),
        ((-50, -100, 50), -1.0f32),
        ((-50, -100, 100), -1.0f32),
        ((-50, 0, -100), -1.0f32),
        ((-50, 0, -50), -1.0f32),
        ((-50, 0, 0), -1.0f32),
        ((-50, 0, 50), -1.0f32),
        ((-50, 0, 100), -1.0f32),
        ((-50, 100, -100), -1.0f32),
        ((-50, 100, -50), -1.0f32),
        ((-50, 100, 0), -1.0f32),
        ((-50, 100, 50), -1.0f32),
        ((-50, 100, 100), -1.0f32),
        ((-50, 200, -100), -1.0f32),
        ((-50, 200, -50), -1.0f32),
        ((-50, 200, 0), -1.0f32),
        ((-50, 200, 50), -1.0f32),
        ((-50, 200, 100), -1.0f32),
        ((0, -200, -100), -1.0f32),
        ((0, -200, -50), -1.0f32),
        ((0, -200, 0), -1.0f32),
        ((0, -200, 50), -1.0f32),
        ((0, -200, 100), -1.0f32),
        ((0, -100, -100), -1.0f32),
        ((0, -100, -50), -1.0f32),
        ((0, -100, 0), -1.0f32),
        ((0, -100, 50), -1.0f32),
        ((0, -100, 100), -1.0f32),
        ((0, 0, -100), -1.0f32),
        ((0, 0, -50), -1.0f32),
        ((0, 0, 0), -1.0f32),
        ((0, 0, 50), -1.0f32),
        ((0, 0, 100), -0.37225938f32),
        ((0, 100, -100), -1.0f32),
        ((0, 100, -50), -1.0f32),
        ((0, 100, 0), -1.0f32),
        ((0, 100, 50), -1.0f32),
        ((0, 100, 100), -1.0f32),
        ((0, 200, -100), -1.0f32),
        ((0, 200, -50), -1.0f32),
        ((0, 200, 0), -1.0f32),
        ((0, 200, 50), -1.0f32),
        ((0, 200, 100), -1.0f32),
        ((50, -200, -100), -1.0f32),
        ((50, -200, -50), -1.0f32),
        ((50, -200, 0), -1.0f32),
        ((50, -200, 50), -1.0f32),
        ((50, -200, 100), -1.0f32),
        ((50, -100, -100), -1.0f32),
        ((50, -100, -50), -1.0f32),
        ((50, -100, 0), -1.0f32),
        ((50, -100, 50), -1.0f32),
        ((50, -100, 100), -1.0f32),
        ((50, 0, -100), -1.0f32),
        ((50, 0, -50), -1.0f32),
        ((50, 0, 0), -1.0f32),
        ((50, 0, 50), -1.0f32),
        ((50, 0, 100), -1.0f32),
        ((50, 100, -100), -1.0f32),
        ((50, 100, -50), -1.0f32),
        ((50, 100, 0), -1.0f32),
        ((50, 100, 50), -1.0f32),
        ((50, 100, 100), -1.0f32),
        ((50, 200, -100), -1.0f32),
        ((50, 200, -50), -1.0f32),
        ((50, 200, 0), -1.0f32),
        ((50, 200, 50), -1.0f32),
        ((50, 200, 100), -1.0f32),
        ((100, -200, -100), -1.0f32),
        ((100, -200, -50), -1.0f32),
        ((100, -200, 0), -1.0f32),
        ((100, -200, 50), -1.0f32),
        ((100, -200, 100), -1.0f32),
        ((100, -100, -100), -1.0f32),
        ((100, -100, -50), -1.0f32),
        ((100, -100, 0), -1.0f32),
        ((100, -100, 50), -1.0f32),
        ((100, -100, 100), -1.0f32),
        ((100, 0, -100), -0.5633548f32),
        ((100, 0, -50), -1.0f32),
        ((100, 0, 0), -1.0f32),
        ((100, 0, 50), -1.0f32),
        ((100, 0, 100), -1.0f32),
        ((100, 100, -100), -1.0f32),
        ((100, 100, -50), -1.0f32),
        ((100, 100, 0), -1.0f32),
        ((100, 100, 50), -1.0f32),
        ((100, 100, 100), -1.0f32),
        ((100, 200, -100), -1.0f32),
        ((100, 200, -50), -1.0f32),
        ((100, 200, 0), -1.0f32),
        ((100, 200, 50), -1.0f32),
        ((100, 200, 100), -1.0f32),
    ];
    for ((x, y, z), value) in values {
        let pos = Vector3 { x, y, z };
        assert_eq!(sample_noise_router_function!(vein_ridged, pos), value);
    }

    let values = [
        ((-100, -200, -100), -0.6211142f32),
        ((-100, -200, -50), -0.39648867f32),
        ((-100, -200, 0), 0.13614774f32),
        ((-100, -200, 50), -0.16959935f32),
        ((-100, -200, 100), -0.32338807f32),
        ((-100, -100, -100), 0.19360241f32),
        ((-100, -100, -50), -0.5524223f32),
        ((-100, -100, 0), -0.57981896f32),
        ((-100, -100, 50), 0.07911202f32),
        ((-100, -100, 100), -0.69137764f32),
        ((-100, 0, -100), -0.3517989f32),
        ((-100, 0, -50), -0.2316089f32),
        ((-100, 0, 0), -0.71462065f32),
        ((-100, 0, 50), -0.11191794f32),
        ((-100, 0, 100), -0.50183684f32),
        ((-100, 100, -100), 0.020010382f32),
        ((-100, 100, -50), -0.16182442f32),
        ((-100, 100, 0), 0.1810107f32),
        ((-100, 100, 50), -0.05977027f32),
        ((-100, 100, 100), -0.38239762f32),
        ((-100, 200, -100), -0.31822476f32),
        ((-100, 200, -50), -0.38691443f32),
        ((-100, 200, 0), -0.46208096f32),
        ((-100, 200, 50), -0.14308953f32),
        ((-100, 200, 100), -0.3662827f32),
        ((-50, -200, -100), -0.59444964f32),
        ((-50, -200, -50), -0.021772116f32),
        ((-50, -200, 0), -0.4553607f32),
        ((-50, -200, 50), -0.7361001f32),
        ((-50, -200, 100), -0.31090656f32),
        ((-50, -100, -100), -0.21794732f32),
        ((-50, -100, -50), -0.016295493f32),
        ((-50, -100, 0), -0.38851517f32),
        ((-50, -100, 50), -0.51999193f32),
        ((-50, -100, 100), 0.11613488f32),
        ((-50, 0, -100), -0.5138435f32),
        ((-50, 0, -50), -0.017523468f32),
        ((-50, 0, 0), 0.1954177f32),
        ((-50, 0, 50), -0.19536033f32),
        ((-50, 0, 100), -0.34434137f32),
        ((-50, 100, -100), -0.67770505f32),
        ((-50, 100, -50), -0.43711895f32),
        ((-50, 100, 0), -0.07361551f32),
        ((-50, 100, 50), -0.19442755f32),
        ((-50, 100, 100), -0.110158816f32),
        ((-50, 200, -100), -0.50939107f32),
        ((-50, 200, -50), -0.21223885f32),
        ((-50, 200, 0), -0.50843954f32),
        ((-50, 200, 50), 0.2814808f32),
        ((-50, 200, 100), 0.07975656f32),
        ((0, -200, -100), -0.57179856f32),
        ((0, -200, -50), -0.46521252f32),
        ((0, -200, 0), -0.48324388f32),
        ((0, -200, 50), -0.01284042f32),
        ((0, -200, 100), -0.11899963f32),
        ((0, -100, -100), -0.2023485f32),
        ((0, -100, -50), -0.122145385f32),
        ((0, -100, 0), -0.40598124f32),
        ((0, -100, 50), -0.7050743f32),
        ((0, -100, 100), 0.21016705f32),
        ((0, 0, -100), -0.173097f32),
        ((0, 0, -50), -0.015652657f32),
        ((0, 0, 0), -0.7566469f32),
        ((0, 0, 50), -0.113117784f32),
        ((0, 0, 100), -0.23832245f32),
        ((0, 100, -100), -0.33280423f32),
        ((0, 100, -50), -0.48286933f32),
        ((0, 100, 0), -0.19238818f32),
        ((0, 100, 50), -0.09430514f32),
        ((0, 100, 100), 0.3641494f32),
        ((0, 200, -100), -0.008374274f32),
        ((0, 200, -50), -0.6089201f32),
        ((0, 200, 0), -0.19137877f32),
        ((0, 200, 50), 0.23142749f32),
        ((0, 200, 100), -0.11576079f32),
        ((50, -200, -100), -0.10558416f32),
        ((50, -200, -50), -0.067754686f32),
        ((50, -200, 0), -0.2325832f32),
        ((50, -200, 50), -0.18825895f32),
        ((50, -200, 100), -0.10597208f32),
        ((50, -100, -100), 0.07295096f32),
        ((50, -100, -50), 0.29925054f32),
        ((50, -100, 0), 0.06411937f32),
        ((50, -100, 50), -0.22191198f32),
        ((50, -100, 100), -0.50539005f32),
        ((50, 0, -100), -0.8068819f32),
        ((50, 0, -50), -0.50126964f32),
        ((50, 0, 0), -0.8785118f32),
        ((50, 0, 50), -1.2255795f32),
        ((50, 0, 100), 0.0041258633f32),
        ((50, 100, -100), -0.71286976f32),
        ((50, 100, -50), -0.08304784f32),
        ((50, 100, 0), -0.5255188f32),
        ((50, 100, 50), -0.14814368f32),
        ((50, 100, 100), 0.038000733f32),
        ((50, 200, -100), -0.17379472f32),
        ((50, 200, -50), -0.11321899f32),
        ((50, 200, 0), -0.25701085f32),
        ((50, 200, 50), 0.059371352f32),
        ((50, 200, 100), -0.20324698f32),
        ((100, -200, -100), -0.31694457f32),
        ((100, -200, -50), -0.08550918f32),
        ((100, -200, 0), 0.1864973f32),
        ((100, -200, 50), -0.1791727f32),
        ((100, -200, 100), -0.45105514f32),
        ((100, -100, -100), 0.120147884f32),
        ((100, -100, -50), -0.5504334f32),
        ((100, -100, 0), -0.78364074f32),
        ((100, -100, 50), -0.20160359f32),
        ((100, -100, 100), 0.41181856f32),
        ((100, 0, -100), 0.15298164f32),
        ((100, 0, -50), -0.61954427f32),
        ((100, 0, 0), 0.016964585f32),
        ((100, 0, 50), -0.20914406f32),
        ((100, 0, 100), -0.114642024f32),
        ((100, 100, -100), -0.51432776f32),
        ((100, 100, -50), 0.017123312f32),
        ((100, 100, 0), -0.04397598f32),
        ((100, 100, 50), -0.20419465f32),
        ((100, 100, 100), -0.2007871f32),
        ((100, 200, -100), -0.71460867f32),
        ((100, 200, -50), -0.7415182f32),
        ((100, 200, 0), -0.42050377f32),
        ((100, 200, 50), 0.42144108f32),
        ((100, 200, 100), -0.6867497f32),
    ];
    for ((x, y, z), value) in values {
        let pos = Vector3 { x, y, z };
        assert_eq!(sample_noise_router_function!(vein_gap, pos), value);
    }
}

// #[test]
// fn config_final_density() {
//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/final_density_dump_7_4.json");

//     let router = &OVERWORLD_BASE_NOISE_ROUTER.noise;
//     let proto_stack =
//         ProtoNoiseRouters::generate_proto_stack(router.full_component_stack, &RANDOM_CONFIG);
//     let mut stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(&mut stack[..router.final_density]);

//     // This is a lot of data it iter over, one two skip a few done
//     for (x, y, z, sample) in expected_data.into_iter().step_by(5) {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// // The following test the validity density function components of the density functions used in
// // terrain generation. Basically, this aides in narrowing down where errors occur with correctness.

// #[derive(Deserialize)]
// struct DensityFunctionReprs {
//     #[serde(rename = "overworld/base_3d_noise")]
//     base_3d_noise: DensityFunctionRepr,
//     #[serde(rename = "overworld/caves/spaghetti_2d_thickness_modulator")]
//     spaghetti_2d_thickness: DensityFunctionRepr,
//     #[serde(rename = "overworld/caves/pillars")]
//     cave_pillars: DensityFunctionRepr,
//     #[serde(rename = "overworld/caves/noodle")]
//     cave_noodle: DensityFunctionRepr,
//     #[serde(rename = "overworld/caves/spaghetti_roughness_function")]
//     spaghetti_roughness: DensityFunctionRepr,
//     #[serde(rename = "overworld/caves/entrances")]
//     cave_entrances: DensityFunctionRepr,
//     #[serde(rename = "overworld/caves/spaghetti_2d")]
//     spaghetti_2d: DensityFunctionRepr,
//     #[serde(rename = "overworld/offset")]
//     offset: DensityFunctionRepr,
//     #[serde(rename = "overworld/depth")]
//     depth: DensityFunctionRepr,
//     #[serde(rename = "overworld/factor")]
//     factor: DensityFunctionRepr,
//     #[serde(rename = "overworld/sloped_cheese")]
//     sloped_cheese: DensityFunctionRepr,
// }

// macro_rules! read_data_from_file_json5 {
//     ($path:expr) => {
//         serde_json5::from_str(
//             &fs::read_to_string(
//                 Path::new(env!("CARGO_MANIFEST_DIR"))
//                     .parent()
//                     .unwrap()
//                     .join(file!())
//                     .parent()
//                     .unwrap()
//                     .join($path),
//             )
//             .expect("no data file"),
//         )
//         .expect("failed to decode data")
//     };
// }

// static DENSITY_FUNCTION_REPRS: LazyLock<DensityFunctionReprs> = LazyLock::new(|| {
//     read_data_from_file_json5!("../../../../../assets/density_function_tests.json")
// });

// #[test]
// fn base_sloped_cheese() {
//     let base_stack = DENSITY_FUNCTION_REPRS.sloped_cheese.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_sloped_cheese_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_factor() {
//     let base_stack = DENSITY_FUNCTION_REPRS.factor.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_factor_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_depth() {
//     let base_stack = DENSITY_FUNCTION_REPRS.depth.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_depth_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_offset() {
//     let base_stack = DENSITY_FUNCTION_REPRS.offset.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_offset_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_cave_entrances() {
//     let base_stack = DENSITY_FUNCTION_REPRS.cave_entrances.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_cave_entrances_overworld_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_3d_noise() {
//     let base_stack = DENSITY_FUNCTION_REPRS.base_3d_noise.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_3d_overworld_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_spahetti_roughness() {
//     let base_stack = DENSITY_FUNCTION_REPRS
//         .spaghetti_roughness
//         .base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> = read_data_from_file!(
//         "../../../../../assets/converted_cave_spaghetti_rough_overworld_7_4.json"
//     );
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_cave_noodle() {
//     let base_stack = DENSITY_FUNCTION_REPRS.cave_noodle.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_cave_noodle_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_cave_pillars() {
//     let base_stack = DENSITY_FUNCTION_REPRS.cave_pillars.base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_cave_pillar_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

// #[test]
// fn base_spaghetti_2d_thickness() {
//     let base_stack = DENSITY_FUNCTION_REPRS
//         .spaghetti_2d_thickness
//         .base_component_stack();
//     let proto_stack = ProtoNoiseRouters::generate_proto_stack(&base_stack, &RANDOM_CONFIG);
//     let mut function_stack = build_function_stack!(proto_stack);
//     let mut function = build_function!(function_stack);

//     let expected_data: Vec<(i32, i32, i32, f32)> =
//         read_data_from_file!("../../../../../assets/converted_cave_spaghetti_2d_thicc_7_4.json");
//     for (x, y, z, sample) in expected_data {
//         let pos = Vector3 { x, y, z };
//         assert_eq_delta!(
//             function.sample(&pos, &TEST_SAMPLE_OPTIONS),
//             sample,
//             f32::EPSILON
//         );
//     }
// }

#[test]
fn test_round_function_parity() {
    use super::math::round_to_integer;
    use pumpkin_data::noise_router::RoundingOperation;

    // Type 0: Floor
    assert_eq!(round_to_integer(1.7, RoundingOperation::Floor), 1.0);
    assert_eq!(round_to_integer(-1.7, RoundingOperation::Floor), -2.0);
    assert_eq!(round_to_integer(1.0, RoundingOperation::Floor), 1.0);
    assert_eq!(round_to_integer(0.0, RoundingOperation::Floor), 0.0);

    // Type 1: Round ((float)Math.round(x) in Java is floor(x + 0.5))
    assert_eq!(round_to_integer(1.2, RoundingOperation::Round), 1.0);
    assert_eq!(round_to_integer(1.5, RoundingOperation::Round), 2.0);
    assert_eq!(round_to_integer(1.7, RoundingOperation::Round), 2.0);
    assert_eq!(round_to_integer(-1.2, RoundingOperation::Round), -1.0);
    assert_eq!(round_to_integer(-1.5, RoundingOperation::Round), -1.0);
    assert_eq!(round_to_integer(-1.7, RoundingOperation::Round), -2.0);

    // Type 2: Ceil
    assert_eq!(round_to_integer(1.2, RoundingOperation::Ceil), 2.0);
    assert_eq!(round_to_integer(-1.7, RoundingOperation::Ceil), -1.0);
    assert_eq!(round_to_integer(1.0, RoundingOperation::Ceil), 1.0);

    // Type 3: Truncate (x > 0 ? floor(x) : ceil(x))
    assert_eq!(round_to_integer(1.7, RoundingOperation::Truncate), 1.0);
    assert_eq!(round_to_integer(-1.7, RoundingOperation::Truncate), -1.0);
    assert_eq!(round_to_integer(0.0, RoundingOperation::Truncate), 0.0);
}

#[test]
fn test_gradient_function_parity() {
    use super::StaticIndependentChunkNoiseFunctionComponentImpl;
    use super::misc::Gradient;
    use pumpkin_data::noise_router::{Axis, GradientData, Tiling};

    // Clamped sampler: from_y = 0, to_y = 10, from_val = 0.0, to_val = 100.0
    static CLAMPED_DATA: GradientData = GradientData {
        axis: Axis::Y,
        tiling: Tiling::ClampToEdge,
        from_coordinate: 0,
        to_coordinate: 10,
        from_value: 0.0,
        to_value: 100.0,
    };
    let clamped = Gradient::new(&CLAMPED_DATA);
    assert_eq!(clamped.sample(&Vector3::new(0, -5, 0)), 0.0);
    assert_eq!(clamped.sample(&Vector3::new(0, 0, 0)), 0.0);
    assert_eq!(clamped.sample(&Vector3::new(0, 5, 0)), 50.0);
    assert_eq!(clamped.sample(&Vector3::new(0, 10, 0)), 100.0);
    assert_eq!(clamped.sample(&Vector3::new(0, 15, 0)), 100.0);

    // Repeat sampler: range 10
    static REPEAT_DATA: GradientData = GradientData {
        axis: Axis::Y,
        tiling: Tiling::Repeat,
        from_coordinate: 0,
        to_coordinate: 10,
        from_value: 0.0,
        to_value: 100.0,
    };
    let repeat = Gradient::new(&REPEAT_DATA);
    assert_eq!(repeat.sample(&Vector3::new(0, 0, 0)), 0.0);
    assert_eq!(repeat.sample(&Vector3::new(0, 5, 0)), 50.0);
    assert_eq!(repeat.sample(&Vector3::new(0, 10, 0)), 0.0);
    assert_eq!(repeat.sample(&Vector3::new(0, 15, 0)), 50.0);
    assert_eq!(repeat.sample(&Vector3::new(0, -5, 0)), 50.0);

    // MirroredRepeat sampler: range 10
    static MIRRORED_DATA: GradientData = GradientData {
        axis: Axis::Y,
        tiling: Tiling::MirroredRepeat,
        from_coordinate: 0,
        to_coordinate: 10,
        from_value: 0.0,
        to_value: 100.0,
    };
    let mirrored = Gradient::new(&MIRRORED_DATA);
    assert_eq!(mirrored.sample(&Vector3::new(0, 0, 0)), 0.0);
    assert_eq!(mirrored.sample(&Vector3::new(0, 5, 0)), 50.0);
    assert_eq!(mirrored.sample(&Vector3::new(0, 10, 0)), 100.0);
    assert_eq!(mirrored.sample(&Vector3::new(0, 15, 0)), 50.0);
    assert_eq!(mirrored.sample(&Vector3::new(0, 20, 0)), 0.0);
    assert_eq!(mirrored.sample(&Vector3::new(0, -5, 0)), 50.0);
}
