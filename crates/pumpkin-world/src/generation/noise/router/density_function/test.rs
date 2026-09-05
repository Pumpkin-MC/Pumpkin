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
use crate::generation::noise::router::chunk_noise_router::{
    ChunkNoiseDensityFunction, ChunkNoiseFunctionComponent,
};
use crate::generation::noise::router::proto_noise_router::{
    ProtoNoiseFunctionComponent, ProtoNoiseRouters,
};

use super::{NoiseFunctionComponentRange, PassThrough};

// This is a dummy value because we are not actually building chunk-specific functions
static TEST_OPTIONS: ChunkNoiseFunctionBuilderOptions =
    ChunkNoiseFunctionBuilderOptions::new(Vec::new(), Vec::new(), None);
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

macro_rules! sample_noise_router_function {
    ($name:ident, $pos: expr) => {{
        let base_router = &OVERWORLD_BASE_NOISE_ROUTER.noise;
        let proto_stack = ProtoNoiseRouters::generate_proto_stack(
            &base_router.full_component_stack,
            &RANDOM_CONFIG,
        );
        let mut stack = build_function_stack!(&proto_stack[..=base_router.$name]);
        let mut function = build_function!(&mut stack);
        function.sample(&$pos)
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
        function.sample(&$pos)
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
        function.sample(&$pos)
    }};
}

// TODO: Test all dimensions/noise routers

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= expected.abs().max(1.0) * 1e-5,
        "{actual} is not close to {expected}"
    );
}

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
    assert_close(
        sample_noise_router_function!(barrier_noise, pos),
        -0.54002273f32,
    );
    assert_close(
        sample_noise_router_function!(fluid_level_floodedness_noise, pos),
        -0.4709572f32,
    );
    assert_close(
        sample_noise_router_function!(fluid_level_spread_noise, pos),
        -0.05726914f32,
    );
    assert_close(
        sample_noise_router_function!(lava_noise, pos),
        -0.16423604f32,
    );
    assert_close(
        sample_multi_noise_router_function!(temperature, pos),
        0.11823799f32,
    );
    assert_close(
        sample_multi_noise_router_function!(vegetation, pos),
        -0.0013601681f32,
    );
    assert_close(
        sample_multi_noise_router_function!(continents, pos),
        -0.008171953f32,
    );
    assert_close(
        sample_multi_noise_router_function!(erosion, pos),
        -0.10391074f32,
    );
    assert_close(
        sample_multi_noise_router_function!(depth, pos),
        0.4118821f32,
    );
    assert_close(
        sample_multi_noise_router_function!(ridges, pos),
        0.011110324f32,
    );
    assert_close(sample_surface_router_function!(pos), 40.0f32);
    assert_close(
        sample_noise_router_function!(final_density, pos),
        0.15719144f32,
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
        ((-100, 0, -100), 0.34622896f32),
        ((-100, 0, -50), 0.23404458f32),
        ((-100, 0, 0), -0.028825896f32),
        ((-100, 0, 50), -0.16684759f32),
        ((-100, 0, 100), -0.18434657f32),
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
        ((-50, 0, -100), 0.057578042f32),
        ((-50, 0, -50), 0.0014520884f32),
        ((-50, 0, 0), -0.024149703f32),
        ((-50, 0, 50), 0.12876187f32),
        ((-50, 0, 100), 0.25507578f32),
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
        ((0, 0, -100), -0.24030946f32),
        ((0, 0, -50), -0.24705146f32),
        ((0, 0, 0), -0.06643492f32),
        ((0, 0, 50), 0.2531865f32),
        ((0, 0, 100), 0.48257518f32),
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
        ((50, 0, -100), 0.035583116f32),
        ((50, 0, -50), -0.07225364f32),
        ((50, 0, 0), -0.034741163f32),
        ((50, 0, 50), 0.12616423f32),
        ((50, 0, 100), 0.3541485f32),
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
        ((100, 0, -100), 0.41514847f32),
        ((100, 0, -50), 0.20926304f32),
        ((100, 0, 0), -0.00992015f32),
        ((100, 0, 50), -0.14997281f32),
        ((100, 0, 100), -0.057775974f32),
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
        assert_close(sample_noise_router_function!(vein_toggle, pos), *value);
    }

    let values = [
        ((-100, -200, -100), -0.08f32),
        ((-100, -200, -50), -0.08f32),
        ((-100, -200, 0), -0.08f32),
        ((-100, -200, 50), -0.08f32),
        ((-100, -200, 100), -0.08f32),
        ((-100, -100, -100), -0.08f32),
        ((-100, -100, -50), -0.08f32),
        ((-100, -100, 0), -0.08f32),
        ((-100, -100, 50), -0.08f32),
        ((-100, -100, 100), -0.08f32),
        ((-100, 0, -100), 0.20661108f32),
        ((-100, 0, -50), 0.13701288f32),
        ((-100, 0, 0), 0.7331012f32),
        ((-100, 0, 50), 0.58878726f32),
        ((-100, 0, 100), 0.022846147f32),
        ((-100, 100, -100), -0.08f32),
        ((-100, 100, -50), -0.08f32),
        ((-100, 100, 0), -0.08f32),
        ((-100, 100, 50), -0.08f32),
        ((-100, 100, 100), -0.08f32),
        ((-100, 200, -100), -0.08f32),
        ((-100, 200, -50), -0.08f32),
        ((-100, 200, 0), -0.08f32),
        ((-100, 200, 50), -0.08f32),
        ((-100, 200, 100), -0.08f32),
        ((-50, -200, -100), -0.08f32),
        ((-50, -200, -50), -0.08f32),
        ((-50, -200, 0), -0.08f32),
        ((-50, -200, 50), -0.08f32),
        ((-50, -200, 100), -0.08f32),
        ((-50, -100, -100), -0.08f32),
        ((-50, -100, -50), -0.08f32),
        ((-50, -100, 0), -0.08f32),
        ((-50, -100, 50), -0.08f32),
        ((-50, -100, 100), -0.08f32),
        ((-50, 0, -100), 0.35588402f32),
        ((-50, 0, -50), 0.18297194f32),
        ((-50, 0, 0), 0.087047145f32),
        ((-50, 0, 50), 0.10449423f32),
        ((-50, 0, 100), 0.5929749f32),
        ((-50, 100, -100), -0.08f32),
        ((-50, 100, -50), -0.08f32),
        ((-50, 100, 0), -0.08f32),
        ((-50, 100, 50), -0.08f32),
        ((-50, 100, 100), -0.08f32),
        ((-50, 200, -100), -0.08f32),
        ((-50, 200, -50), -0.08f32),
        ((-50, 200, 0), -0.08f32),
        ((-50, 200, 50), -0.08f32),
        ((-50, 200, 100), -0.08f32),
        ((0, -200, -100), -0.08f32),
        ((0, -200, -50), -0.08f32),
        ((0, -200, 0), -0.08f32),
        ((0, -200, 50), -0.08f32),
        ((0, -200, 100), -0.08f32),
        ((0, -100, -100), -0.08f32),
        ((0, -100, -50), -0.08f32),
        ((0, -100, 0), -0.08f32),
        ((0, -100, 50), -0.08f32),
        ((0, -100, 100), -0.08f32),
        ((0, 0, -100), 0.35314775f32),
        ((0, 0, -50), 0.1564918f32),
        ((0, 0, 0), 0.5716364f32),
        ((0, 0, 50), 0.28359294f32),
        ((0, 0, 100), 0.3722601f32),
        ((0, 100, -100), -0.08f32),
        ((0, 100, -50), -0.08f32),
        ((0, 100, 0), -0.08f32),
        ((0, 100, 50), -0.08f32),
        ((0, 100, 100), -0.08f32),
        ((0, 200, -100), -0.08f32),
        ((0, 200, -50), -0.08f32),
        ((0, 200, 0), -0.08f32),
        ((0, 200, 50), -0.08f32),
        ((0, 200, 100), -0.08f32),
        ((50, -200, -100), -0.08f32),
        ((50, -200, -50), -0.08f32),
        ((50, -200, 0), -0.08f32),
        ((50, -200, 50), -0.08f32),
        ((50, -200, 100), -0.08f32),
        ((50, -100, -100), -0.08f32),
        ((50, -100, -50), -0.08f32),
        ((50, -100, 0), -0.08f32),
        ((50, -100, 50), -0.08f32),
        ((50, -100, 100), -0.08f32),
        ((50, 0, -100), 0.17334639f32),
        ((50, 0, -50), 0.33464414f32),
        ((50, 0, 0), 0.26210386f32),
        ((50, 0, 50), 0.15279068f32),
        ((50, 0, 100), 0.41071093f32),
        ((50, 100, -100), -0.08f32),
        ((50, 100, -50), -0.08f32),
        ((50, 100, 0), -0.08f32),
        ((50, 100, 50), -0.08f32),
        ((50, 100, 100), -0.08f32),
        ((50, 200, -100), -0.08f32),
        ((50, 200, -50), -0.08f32),
        ((50, 200, 0), -0.08f32),
        ((50, 200, 50), -0.08f32),
        ((50, 200, 100), -0.08f32),
        ((100, -200, -100), -0.08f32),
        ((100, -200, -50), -0.08f32),
        ((100, -200, 0), -0.08f32),
        ((100, -200, 50), -0.08f32),
        ((100, -200, 100), -0.08f32),
        ((100, -100, -100), -0.08f32),
        ((100, -100, -50), -0.08f32),
        ((100, -100, 0), -0.08f32),
        ((100, -100, 50), -0.08f32),
        ((100, -100, 100), -0.08f32),
        ((100, 0, -100), 0.5633548f32),
        ((100, 0, -50), 0.09284274f32),
        ((100, 0, 0), 0.36438537f32),
        ((100, 0, 50), 0.20350628f32),
        ((100, 0, 100), 0.34207004f32),
        ((100, 100, -100), -0.08f32),
        ((100, 100, -50), -0.08f32),
        ((100, 100, 0), -0.08f32),
        ((100, 100, 50), -0.08f32),
        ((100, 100, 100), -0.08f32),
        ((100, 200, -100), -0.08f32),
        ((100, 200, -50), -0.08f32),
        ((100, 200, 0), -0.08f32),
        ((100, 200, 50), -0.08f32),
        ((100, 200, 100), -0.08f32),
    ];
    for ((x, y, z), value) in values {
        let pos = Vector3 { x, y, z };
        assert_close(sample_noise_router_function!(vein_ridged, pos), value);
    }

    let values = [
        ((-100, -200, -100), 0.3211144f32),
        ((-100, -200, -50), 0.0964887f32),
        ((-100, -200, 0), -0.43614775f32),
        ((-100, -200, 50), -0.13040078f32),
        ((-100, -200, 100), 0.023388118f32),
        ((-100, -100, -100), -0.49360234f32),
        ((-100, -100, -50), 0.25242236f32),
        ((-100, -100, 0), 0.27981898f32),
        ((-100, -100, 50), -0.37911198f32),
        ((-100, -100, 100), 0.39137757f32),
        ((-100, 0, -100), 0.05179885f32),
        ((-100, 0, -50), -0.068391114f32),
        ((-100, 0, 0), 0.4146206f32),
        ((-100, 0, 50), -0.18808192f32),
        ((-100, 0, 100), 0.20183684f32),
        ((-100, 100, -100), -0.32001048f32),
        ((-100, 100, -50), -0.13817562f32),
        ((-100, 100, 0), -0.48101068f32),
        ((-100, 100, 50), -0.24022967f32),
        ((-100, 100, 100), 0.08239766f32),
        ((-100, 200, -100), 0.018224742f32),
        ((-100, 200, -50), 0.086914346f32),
        ((-100, 200, 0), 0.162081f32),
        ((-100, 200, 50), -0.15691054f32),
        ((-100, 200, 100), 0.066282704f32),
        ((-50, -200, -100), 0.29444972f32),
        ((-50, -200, -50), -0.27822798f32),
        ((-50, -200, 0), 0.15536064f32),
        ((-50, -200, 50), 0.43609995f32),
        ((-50, -200, 100), 0.010906473f32),
        ((-50, -100, -100), -0.082052454f32),
        ((-50, -100, -50), -0.28370452f32),
        ((-50, -100, 0), 0.088514976f32),
        ((-50, -100, 50), 0.21999179f32),
        ((-50, -100, 100), -0.41613498f32),
        ((-50, 0, -100), 0.2138434f32),
        ((-50, 0, -50), -0.2824767f32),
        ((-50, 0, 0), -0.49541783f32),
        ((-50, 0, 50), -0.104639694f32),
        ((-50, 0, 100), 0.044341385f32),
        ((-50, 100, -100), 0.37770486f32),
        ((-50, 100, -50), 0.13711911f32),
        ((-50, 100, 0), -0.22638431f32),
        ((-50, 100, 50), -0.10557245f32),
        ((-50, 100, 100), -0.18984118f32),
        ((-50, 200, -100), 0.20939098f32),
        ((-50, 200, -50), -0.08776104f32),
        ((-50, 200, 0), 0.20843944f32),
        ((-50, 200, 50), -0.58148074f32),
        ((-50, 200, 100), -0.37975648f32),
        ((0, -200, -100), 0.27179852f32),
        ((0, -200, -50), 0.16521263f32),
        ((0, -200, 0), 0.18324393f32),
        ((0, -200, 50), -0.28715953f32),
        ((0, -200, 100), -0.18100026f32),
        ((0, -100, -100), -0.09765134f32),
        ((0, -100, -50), -0.17785472f32),
        ((0, -100, 0), 0.105981186f32),
        ((0, -100, 50), 0.40507433f32),
        ((0, -100, 100), -0.5101671f32),
        ((0, 0, -100), -0.126903f32),
        ((0, 0, -50), -0.28434744f32),
        ((0, 0, 0), 0.45664692f32),
        ((0, 0, 50), -0.18688229f32),
        ((0, 0, 100), -0.061677486f32),
        ((0, 100, -100), 0.03280422f32),
        ((0, 100, -50), 0.18286934f32),
        ((0, 100, 0), -0.10761196f32),
        ((0, 100, 50), -0.20569485f32),
        ((0, 100, 100), -0.6641494f32),
        ((0, 200, -100), -0.29162562f32),
        ((0, 200, -50), 0.30892023f32),
        ((0, 200, 0), -0.10862115f32),
        ((0, 200, 50), -0.53142744f32),
        ((0, 200, 100), -0.18423921f32),
        ((50, -200, -100), -0.19441606f32),
        ((50, -200, -50), -0.2322452f32),
        ((50, -200, 0), -0.06741692f32),
        ((50, -200, 50), -0.1117409f32),
        ((50, -200, 100), -0.19402796f32),
        ((50, -100, -100), -0.37295103f32),
        ((50, -100, -50), -0.59925056f32),
        ((50, -100, 0), -0.36411932f32),
        ((50, -100, 50), -0.07808812f32),
        ((50, -100, 100), 0.20539f32),
        ((50, 0, -100), 0.50688195f32),
        ((50, 0, -50), 0.20126973f32),
        ((50, 0, 0), 0.57851183f32),
        ((50, 0, 50), 0.9255793f32),
        ((50, 0, 100), -0.30412588f32),
        ((50, 100, -100), 0.4128697f32),
        ((50, 100, -50), -0.21695228f32),
        ((50, 100, 0), 0.22551876f32),
        ((50, 100, 50), -0.1518563f32),
        ((50, 100, 100), -0.33800077f32),
        ((50, 200, -100), -0.12620535f32),
        ((50, 200, -50), -0.1867812f32),
        ((50, 200, 0), -0.042989194f32),
        ((50, 200, 50), -0.35937142f32),
        ((50, 200, 100), -0.096752904f32),
        ((100, -200, -100), 0.016944328f32),
        ((100, -200, -50), -0.21449101f32),
        ((100, -200, 0), -0.48649755f32),
        ((100, -200, 50), -0.120827615f32),
        ((100, -200, 100), 0.15105501f32),
        ((100, -100, -100), -0.42014802f32),
        ((100, -100, -50), 0.25043356f32),
        ((100, -100, 0), 0.4836408f32),
        ((100, -100, 50), -0.09839639f32),
        ((100, -100, 100), -0.7118186f32),
        ((100, 0, -100), -0.45298177f32),
        ((100, 0, -50), 0.31954432f32),
        ((100, 0, 0), -0.31696457f32),
        ((100, 0, 50), -0.090856135f32),
        ((100, 0, 100), -0.18535799f32),
        ((100, 100, -100), 0.21432759f32),
        ((100, 100, -50), -0.31712332f32),
        ((100, 100, 0), -0.25602394f32),
        ((100, 100, 50), -0.09580542f32),
        ((100, 100, 100), -0.099212855f32),
        ((100, 200, -100), 0.41460875f32),
        ((100, 200, -50), 0.44151843f32),
        ((100, 200, 0), 0.1205035f32),
        ((100, 200, 50), -0.7214411f32),
        ((100, 200, 100), 0.38674968f32),
    ];
    for ((x, y, z), value) in values {
        let pos = Vector3 { x, y, z };
        assert_close(sample_noise_router_function!(vein_gap, pos), value);
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
//             function.sample(&pos),
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
