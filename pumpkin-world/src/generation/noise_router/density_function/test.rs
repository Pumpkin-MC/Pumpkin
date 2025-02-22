use pumpkin_util::assert_eq_delta;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use crate::generation::GlobalRandomConfig;
use crate::generation::noise_router::chunk_density_function::{
    ChunkNoiseFunctionSampleOptions, SampleAction,
};
use crate::generation::noise_router::chunk_noise_router::{
    ChunkNoiseDensityFunction, ChunkNoiseFunctionComponent,
};
use crate::generation::noise_router::proto_noise_router::{
    DoublePerlinNoiseBuilder, ProtoNoiseFunctionComponent, recursive_build_proto_stack,
};
use crate::noise_router::NOISE_ROUTER_ASTS;
use crate::read_data_from_file;
use crate::{
    generation::noise_router::chunk_density_function::ChunkNoiseFunctionBuilderOptions,
    noise_router::density_function_ast::DensityFunctionRepr,
};

use super::{NoisePos, PassThrough};

#[derive(Debug)]
struct TestNoisePos {
    x: i32,
    y: i32,
    z: i32,
}

impl NoisePos for TestNoisePos {
    fn x(&self) -> i32 {
        self.x
    }
    fn y(&self) -> i32 {
        self.y
    }
    fn z(&self) -> i32 {
        self.z
    }
}

// This is a dummy value because we are not actually building chunk-specific functions
static TEST_OPTIONS: ChunkNoiseFunctionBuilderOptions =
    ChunkNoiseFunctionBuilderOptions::new(0, 0, 0, 0, 0, 0, 0);
const SEED: u64 = 0;
static RANDOM_CONFIG: LazyLock<GlobalRandomConfig> =
    LazyLock::new(|| GlobalRandomConfig::new(SEED));

macro_rules! build_proto_stack {
    ($repr:expr) => {{
        let mut stack = Vec::<ProtoNoiseFunctionComponent>::new();
        // Map of AST hash to index in the stack
        let mut map = HashMap::<u64, usize>::new();
        let mut perlin_noise_builder = DoublePerlinNoiseBuilder::new(&RANDOM_CONFIG);
        recursive_build_proto_stack(
            $repr,
            &RANDOM_CONFIG,
            &mut stack,
            &mut map,
            &mut perlin_noise_builder,
        );

        stack
    }};
}

macro_rules! build_function_stack {
    ($stack:expr) => {{
        $stack
            .iter()
            .map(|component| match component {
                ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                    ChunkNoiseFunctionComponent::PassThrough(PassThrough {
                        input_index: wrapper.input_index,
                        min_value: wrapper.min_value,
                        max_value: wrapper.max_value,
                    })
                }
                ProtoNoiseFunctionComponent::PassThrough(pass_through) => {
                    ChunkNoiseFunctionComponent::PassThrough(pass_through.clone())
                }
                ProtoNoiseFunctionComponent::Dependent(dependent) => {
                    ChunkNoiseFunctionComponent::Dependent(dependent)
                }
                ProtoNoiseFunctionComponent::Independent(independent) => {
                    ChunkNoiseFunctionComponent::Independent(independent)
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

macro_rules! sample_router_function {
    ($name:ident, $pos: expr) => {{
        let function_ast = NOISE_ROUTER_ASTS.overworld().$name();
        let proto_stack = build_proto_stack!(function_ast);
        let mut stack = build_function_stack!(proto_stack);
        let mut function = build_function!(stack);
        function.sample(&$pos, &TEST_SAMPLE_OPTIONS)
    }};
}

// TODO: Test all dimensions/noise routers

#[test]
// This test verifys that the generated functions after seed initialization but before chunk
// initialization matches the respected Java values.
//
// This is equivalent to a Java `NoiseRouter` after being passed into `NoiseConfig` but before being
// passed into `ChunkNoiseGenerator`
fn test_normal_surface_noisified() {
    let pos = TestNoisePos { x: 0, y: 0, z: 0 };
    // TODO: Move these values to a file and create an extractor for them
    assert_eq!(
        sample_router_function!(barrier_noise, pos),
        -0.5400227274000677f64
    );
    assert_eq!(
        sample_router_function!(fluid_level_floodedness_noise, pos),
        -0.4709571987777473f64
    );
    assert_eq!(
        sample_router_function!(fluid_level_spread_noise, pos),
        -0.057269139961514365f64
    );
    assert_eq!(
        sample_router_function!(lava_noise, pos),
        -0.16423603877333556f64
    );
    assert_eq!(
        sample_router_function!(temperature, pos),
        0.1182379898645608f64
    );
    assert_eq!(
        sample_router_function!(vegetation, pos),
        -0.0013601677416915584f64
    );
    assert_eq!(
        sample_router_function!(continents, pos),
        -0.008171952121206487f64
    );
    assert_eq!(
        sample_router_function!(erosion, pos),
        -0.10391073889243099f64
    );
    assert_eq!(sample_router_function!(depth, pos), 0.411882147192955f64);
    assert_eq!(
        sample_router_function!(ridges, pos),
        0.011110323612534296f64
    );
    assert_eq!(
        sample_router_function!(initial_density_without_jaggedness, pos),
        7.668311608489972f64
    );
    assert_eq!(
        sample_router_function!(final_density, pos),
        0.15719144891255343f64
    );

    let values = [
        ((-100, -200, -100), 0.0f64),
        ((-100, -200, -50), 0.0f64),
        ((-100, -200, 0), 0.0f64),
        ((-100, -200, 50), 0.0f64),
        ((-100, -200, 100), 0.0f64),
        ((-100, -100, -100), 0.0f64),
        ((-100, -100, -50), 0.0f64),
        ((-100, -100, 0), 0.0f64),
        ((-100, -100, 50), 0.0f64),
        ((-100, -100, 100), 0.0f64),
        ((-100, 0, -100), 0.3462291472930333f64),
        ((-100, 0, -50), 0.2340445906392791f64),
        ((-100, 0, 0), -0.028825983399710407f64),
        ((-100, 0, 50), -0.16684760357850822f64),
        ((-100, 0, 100), -0.1843465939249143f64),
        ((-100, 100, -100), 0.0f64),
        ((-100, 100, -50), 0.0f64),
        ((-100, 100, 0), 0.0f64),
        ((-100, 100, 50), 0.0f64),
        ((-100, 100, 100), 0.0f64),
        ((-100, 200, -100), 0.0f64),
        ((-100, 200, -50), 0.0f64),
        ((-100, 200, 0), 0.0f64),
        ((-100, 200, 50), 0.0f64),
        ((-100, 200, 100), 0.0f64),
        ((-50, -200, -100), 0.0f64),
        ((-50, -200, -50), 0.0f64),
        ((-50, -200, 0), 0.0f64),
        ((-50, -200, 50), 0.0f64),
        ((-50, -200, 100), 0.0f64),
        ((-50, -100, -100), 0.0f64),
        ((-50, -100, -50), 0.0f64),
        ((-50, -100, 0), 0.0f64),
        ((-50, -100, 50), 0.0f64),
        ((-50, -100, 100), 0.0f64),
        ((-50, 0, -100), 0.05757810206373369f64),
        ((-50, 0, -50), 0.0014520730707135465f64),
        ((-50, 0, 0), -0.024149735708339466f64),
        ((-50, 0, 50), 0.1287619466526521f64),
        ((-50, 0, 100), 0.25507593901831094f64),
        ((-50, 100, -100), 0.0f64),
        ((-50, 100, -50), 0.0f64),
        ((-50, 100, 0), 0.0f64),
        ((-50, 100, 50), 0.0f64),
        ((-50, 100, 100), 0.0f64),
        ((-50, 200, -100), 0.0f64),
        ((-50, 200, -50), 0.0f64),
        ((-50, 200, 0), 0.0f64),
        ((-50, 200, 50), 0.0f64),
        ((-50, 200, 100), 0.0f64),
        ((0, -200, -100), 0.0f64),
        ((0, -200, -50), 0.0f64),
        ((0, -200, 0), 0.0f64),
        ((0, -200, 50), 0.0f64),
        ((0, -200, 100), 0.0f64),
        ((0, -100, -100), 0.0f64),
        ((0, -100, -50), 0.0f64),
        ((0, -100, 0), 0.0f64),
        ((0, -100, 50), 0.0f64),
        ((0, -100, 100), 0.0f64),
        ((0, 0, -100), -0.24030906682975775f64),
        ((0, 0, -50), -0.24705110006127165f64),
        ((0, 0, 0), -0.06643453056181631f64),
        ((0, 0, 50), 0.25318680526509063f64),
        ((0, 0, 100), 0.48257536249146743f64),
        ((0, 100, -100), 0.0f64),
        ((0, 100, -50), 0.0f64),
        ((0, 100, 0), 0.0f64),
        ((0, 100, 50), 0.0f64),
        ((0, 100, 100), 0.0f64),
        ((0, 200, -100), 0.0f64),
        ((0, 200, -50), 0.0f64),
        ((0, 200, 0), 0.0f64),
        ((0, 200, 50), 0.0f64),
        ((0, 200, 100), 0.0f64),
        ((50, -200, -100), 0.0f64),
        ((50, -200, -50), 0.0f64),
        ((50, -200, 0), 0.0f64),
        ((50, -200, 50), 0.0f64),
        ((50, -200, 100), 0.0f64),
        ((50, -100, -100), 0.0f64),
        ((50, -100, -50), 0.0f64),
        ((50, -100, 0), 0.0f64),
        ((50, -100, 50), 0.0f64),
        ((50, -100, 100), 0.0f64),
        ((50, 0, -100), 0.035583298926324954f64),
        ((50, 0, -50), -0.07225351839505538f64),
        ((50, 0, 0), -0.03474107481998612f64),
        ((50, 0, 50), 0.12616421777330467f64),
        ((50, 0, 100), 0.35414843965758613f64),
        ((50, 100, -100), 0.0f64),
        ((50, 100, -50), 0.0f64),
        ((50, 100, 0), 0.0f64),
        ((50, 100, 50), 0.0f64),
        ((50, 100, 100), 0.0f64),
        ((50, 200, -100), 0.0f64),
        ((50, 200, -50), 0.0f64),
        ((50, 200, 0), 0.0f64),
        ((50, 200, 50), 0.0f64),
        ((50, 200, 100), 0.0f64),
        ((100, -200, -100), 0.0f64),
        ((100, -200, -50), 0.0f64),
        ((100, -200, 0), 0.0f64),
        ((100, -200, 50), 0.0f64),
        ((100, -200, 100), 0.0f64),
        ((100, -100, -100), 0.0f64),
        ((100, -100, -50), 0.0f64),
        ((100, -100, 0), 0.0f64),
        ((100, -100, 50), 0.0f64),
        ((100, -100, 100), 0.0f64),
        ((100, 0, -100), 0.4151489417623382f64),
        ((100, 0, -50), 0.2092632456905039f64),
        ((100, 0, 0), -0.009920164828456044f64),
        ((100, 0, 50), -0.14997295538707048f64),
        ((100, 0, 100), -0.05777616780034325f64),
        ((100, 100, -100), 0.0f64),
        ((100, 100, -50), 0.0f64),
        ((100, 100, 0), 0.0f64),
        ((100, 100, 50), 0.0f64),
        ((100, 100, 100), 0.0f64),
        ((100, 200, -100), 0.0f64),
        ((100, 200, -50), 0.0f64),
        ((100, 200, 0), 0.0f64),
        ((100, 200, 50), 0.0f64),
        ((100, 200, 100), 0.0f64),
    ];
    for ((x, y, z), value) in values {
        let pos = TestNoisePos { x, y, z };
        assert_eq!(sample_router_function!(vein_toggle, pos), value);
    }

    let values = [
        ((-100, -200, -100), -0.07999999821186066f64),
        ((-100, -200, -50), -0.07999999821186066f64),
        ((-100, -200, 0), -0.07999999821186066f64),
        ((-100, -200, 50), -0.07999999821186066f64),
        ((-100, -200, 100), -0.07999999821186066f64),
        ((-100, -100, -100), -0.07999999821186066f64),
        ((-100, -100, -50), -0.07999999821186066f64),
        ((-100, -100, 0), -0.07999999821186066f64),
        ((-100, -100, 50), -0.07999999821186066f64),
        ((-100, -100, 100), -0.07999999821186066f64),
        ((-100, 0, -100), 0.20661121715107683f64),
        ((-100, 0, -50), 0.13701288573667827f64),
        ((-100, 0, 0), 0.7331011623931737f64),
        ((-100, 0, 50), 0.5887875159446838f64),
        ((-100, 0, 100), 0.022846022407350147f64),
        ((-100, 100, -100), -0.07999999821186066f64),
        ((-100, 100, -50), -0.07999999821186066f64),
        ((-100, 100, 0), -0.07999999821186066f64),
        ((-100, 100, 50), -0.07999999821186066f64),
        ((-100, 100, 100), -0.07999999821186066f64),
        ((-100, 200, -100), -0.07999999821186066f64),
        ((-100, 200, -50), -0.07999999821186066f64),
        ((-100, 200, 0), -0.07999999821186066f64),
        ((-100, 200, 50), -0.07999999821186066f64),
        ((-100, 200, 100), -0.07999999821186066f64),
        ((-50, -200, -100), -0.07999999821186066f64),
        ((-50, -200, -50), -0.07999999821186066f64),
        ((-50, -200, 0), -0.07999999821186066f64),
        ((-50, -200, 50), -0.07999999821186066f64),
        ((-50, -200, 100), -0.07999999821186066f64),
        ((-50, -100, -100), -0.07999999821186066f64),
        ((-50, -100, -50), -0.07999999821186066f64),
        ((-50, -100, 0), -0.07999999821186066f64),
        ((-50, -100, 50), -0.07999999821186066f64),
        ((-50, -100, 100), -0.07999999821186066f64),
        ((-50, 0, -100), 0.35588447391786027f64),
        ((-50, 0, -50), 0.1829719810187267f64),
        ((-50, 0, 0), 0.08704696157012648f64),
        ((-50, 0, 50), 0.1044941912836557f64),
        ((-50, 0, 100), 0.5929743688753312f64),
        ((-50, 100, -100), -0.07999999821186066f64),
        ((-50, 100, -50), -0.07999999821186066f64),
        ((-50, 100, 0), -0.07999999821186066f64),
        ((-50, 100, 50), -0.07999999821186066f64),
        ((-50, 100, 100), -0.07999999821186066f64),
        ((-50, 200, -100), -0.07999999821186066f64),
        ((-50, 200, -50), -0.07999999821186066f64),
        ((-50, 200, 0), -0.07999999821186066f64),
        ((-50, 200, 50), -0.07999999821186066f64),
        ((-50, 200, 100), -0.07999999821186066f64),
        ((0, -200, -100), -0.07999999821186066f64),
        ((0, -200, -50), -0.07999999821186066f64),
        ((0, -200, 0), -0.07999999821186066f64),
        ((0, -200, 50), -0.07999999821186066f64),
        ((0, -200, 100), -0.07999999821186066f64),
        ((0, -100, -100), -0.07999999821186066f64),
        ((0, -100, -50), -0.07999999821186066f64),
        ((0, -100, 0), -0.07999999821186066f64),
        ((0, -100, 50), -0.07999999821186066f64),
        ((0, -100, 100), -0.07999999821186066f64),
        ((0, 0, -100), 0.3531476519454157f64),
        ((0, 0, -50), 0.15649178293218172f64),
        ((0, 0, 0), 0.5716365265208109f64),
        ((0, 0, 50), 0.28359279788952346f64),
        ((0, 0, 100), 0.37225938767638495f64),
        ((0, 100, -100), -0.07999999821186066f64),
        ((0, 100, -50), -0.07999999821186066f64),
        ((0, 100, 0), -0.07999999821186066f64),
        ((0, 100, 50), -0.07999999821186066f64),
        ((0, 100, 100), -0.07999999821186066f64),
        ((0, 200, -100), -0.07999999821186066f64),
        ((0, 200, -50), -0.07999999821186066f64),
        ((0, 200, 0), -0.07999999821186066f64),
        ((0, 200, 50), -0.07999999821186066f64),
        ((0, 200, 100), -0.07999999821186066f64),
        ((50, -200, -100), -0.07999999821186066f64),
        ((50, -200, -50), -0.07999999821186066f64),
        ((50, -200, 0), -0.07999999821186066f64),
        ((50, -200, 50), -0.07999999821186066f64),
        ((50, -200, 100), -0.07999999821186066f64),
        ((50, -100, -100), -0.07999999821186066f64),
        ((50, -100, -50), -0.07999999821186066f64),
        ((50, -100, 0), -0.07999999821186066f64),
        ((50, -100, 50), -0.07999999821186066f64),
        ((50, -100, 100), -0.07999999821186066f64),
        ((50, 0, -100), 0.1733462217252416f64),
        ((50, 0, -50), 0.33464400306517067f64),
        ((50, 0, 0), 0.2621039147343691f64),
        ((50, 0, 50), 0.15279071012957127f64),
        ((50, 0, 100), 0.4107100561510984f64),
        ((50, 100, -100), -0.07999999821186066f64),
        ((50, 100, -50), -0.07999999821186066f64),
        ((50, 100, 0), -0.07999999821186066f64),
        ((50, 100, 50), -0.07999999821186066f64),
        ((50, 100, 100), -0.07999999821186066f64),
        ((50, 200, -100), -0.07999999821186066f64),
        ((50, 200, -50), -0.07999999821186066f64),
        ((50, 200, 0), -0.07999999821186066f64),
        ((50, 200, 50), -0.07999999821186066f64),
        ((50, 200, 100), -0.07999999821186066f64),
        ((100, -200, -100), -0.07999999821186066f64),
        ((100, -200, -50), -0.07999999821186066f64),
        ((100, -200, 0), -0.07999999821186066f64),
        ((100, -200, 50), -0.07999999821186066f64),
        ((100, -200, 100), -0.07999999821186066f64),
        ((100, -100, -100), -0.07999999821186066f64),
        ((100, -100, -50), -0.07999999821186066f64),
        ((100, -100, 0), -0.07999999821186066f64),
        ((100, -100, 50), -0.07999999821186066f64),
        ((100, -100, 100), -0.07999999821186066f64),
        ((100, 0, -100), 0.5633547588776332f64),
        ((100, 0, -50), 0.09284281739909031f64),
        ((100, 0, 0), 0.36438508670444847f64),
        ((100, 0, 50), 0.20350630763687888f64),
        ((100, 0, 100), 0.342069979071766f64),
        ((100, 100, -100), -0.07999999821186066f64),
        ((100, 100, -50), -0.07999999821186066f64),
        ((100, 100, 0), -0.07999999821186066f64),
        ((100, 100, 50), -0.07999999821186066f64),
        ((100, 100, 100), -0.07999999821186066f64),
        ((100, 200, -100), -0.07999999821186066f64),
        ((100, 200, -50), -0.07999999821186066f64),
        ((100, 200, 0), -0.07999999821186066f64),
        ((100, 200, 50), -0.07999999821186066f64),
        ((100, 200, 100), -0.07999999821186066f64),
    ];
    for ((x, y, z), value) in values {
        let pos = TestNoisePos { x, y, z };
        assert_eq!(sample_router_function!(vein_ridged, pos), value);
    }

    let values = [
        ((-100, -200, -100), 0.3211141881942152f64),
        ((-100, -200, -50), 0.09648864932704422f64),
        ((-100, -200, 0), -0.4361477376844327f64),
        ((-100, -200, 50), -0.13040066209600742f64),
        ((-100, -200, 100), 0.023388060633724863f64),
        ((-100, -100, -100), -0.4936024322458776f64),
        ((-100, -100, -50), 0.2524223066605673f64),
        ((-100, -100, 0), 0.2798189678966397f64),
        ((-100, -100, 50), -0.3791120273954761f64),
        ((-100, -100, 100), 0.39137760850669906f64),
        ((-100, 0, -100), 0.05179888245498191f64),
        ((-100, 0, -50), -0.06839110450348797f64),
        ((-100, 0, 0), 0.4146206374440951f64),
        ((-100, 0, 50), -0.1880820750707125f64),
        ((-100, 0, 100), 0.2018368254585623f64),
        ((-100, 100, -100), -0.32001039415713683f64),
        ((-100, 100, -50), -0.13817558469021005f64),
        ((-100, 100, 0), -0.48101070627664044f64),
        ((-100, 100, 50), -0.2402297366726863f64),
        ((-100, 100, 100), 0.08239761934306493f64),
        ((-100, 200, -100), 0.018224734015781025f64),
        ((-100, 200, -50), 0.08691443377020788f64),
        ((-100, 200, 0), 0.16208094523788294f64),
        ((-100, 200, 50), -0.15691048604152458f64),
        ((-100, 200, 100), 0.06628267159017102f64),
        ((-50, -200, -100), 0.2944496267342102f64),
        ((-50, -200, -50), -0.2782278816662622f64),
        ((-50, -200, 0), 0.15536071363878953f64),
        ((-50, -200, 50), 0.43610007125172995f64),
        ((-50, -200, 100), 0.010906558300465366f64),
        ((-50, -100, -100), -0.08205269591250534f64),
        ((-50, -100, -50), -0.28370450958612364f64),
        ((-50, -100, 0), 0.0885151647444476f64),
        ((-50, -100, 50), 0.21999190041491667f64),
        ((-50, -100, 100), -0.41613490183445756f64),
        ((-50, 0, -100), 0.21384346251180444f64),
        ((-50, 0, -50), -0.2824765568109107f64),
        ((-50, 0, 0), -0.4954177161809242f64),
        ((-50, 0, 50), -0.10463968465592202f64),
        ((-50, 0, 100), 0.04434135773500206f64),
        ((-50, 100, -100), 0.37770507600173986f64),
        ((-50, 100, -50), 0.1371189219046899f64),
        ((-50, 100, 0), -0.22638449889692794f64),
        ((-50, 100, 50), -0.10557246185638242f64),
        ((-50, 100, 100), -0.18984119304391683f64),
        ((-50, 200, -100), 0.20939108846156035f64),
        ((-50, 200, -50), -0.08776116132181612f64),
        ((-50, 200, 0), 0.20843954771862513f64),
        ((-50, 200, 50), -0.5814807800404631f64),
        ((-50, 200, 100), -0.3797565621876845f64),
        ((0, -200, -100), 0.27179855614165527f64),
        ((0, -200, -50), 0.16521252240290915f64),
        ((0, -200, 0), 0.18324386151568745f64),
        ((0, -200, 50), -0.28715960497818555f64),
        ((0, -200, 100), -0.18100038230278442f64),
        ((0, -100, -100), -0.09765150029624575f64),
        ((0, -100, -50), -0.17785462076301697f64),
        ((0, -100, 0), 0.10598123320261281f64),
        ((0, -100, 50), 0.40507433937683573f64),
        ((0, -100, 100), -0.5101670875276502f64),
        ((0, 0, -100), -0.12690301253734718f64),
        ((0, 0, -50), -0.2843473512745877f64),
        ((0, 0, 0), 0.4566468364551488f64),
        ((0, 0, 50), -0.1868822216899071f64),
        ((0, 0, 100), -0.06167756316828358f64),
        ((0, 100, -100), 0.03280421216425878f64),
        ((0, 100, -50), 0.1828693088708832f64),
        ((0, 100, 0), -0.10761184293214024f64),
        ((0, 100, 50), -0.2056948693640283f64),
        ((0, 100, 100), -0.6641494135898256f64),
        ((0, 200, -100), -0.2916257499829836f64),
        ((0, 200, -50), 0.3089200762221871f64),
        ((0, 200, 0), -0.10862123905585815f64),
        ((0, 200, 50), -0.5314274903477223f64),
        ((0, 200, 100), -0.18423922562669878f64),
        ((50, -200, -100), -0.19441584981913765f64),
        ((50, -200, -50), -0.23224532903196352f64),
        ((50, -200, 0), -0.06741680955178693f64),
        ((50, -200, 50), -0.11174106180027958f64),
        ((50, -200, 100), -0.19402793406584085f64),
        ((50, -100, -100), -0.3729509731834053f64),
        ((50, -100, -50), -0.5992505452241598f64),
        ((50, -100, 0), -0.3641193668713385f64),
        ((50, -100, 50), -0.0780880308385808f64),
        ((50, -100, 100), 0.20539004653798706f64),
        ((50, 0, -100), 0.5068819044426225f64),
        ((50, 0, -50), 0.2012696212123102f64),
        ((50, 0, 0), 0.578511778875036f64),
        ((50, 0, 50), 0.9255794468686466f64),
        ((50, 0, 100), -0.30412588794463624f64),
        ((50, 100, -100), 0.4128697472440939f64),
        ((50, 100, -50), -0.2169521808135427f64),
        ((50, 100, 0), 0.22551879656869442f64),
        ((50, 100, 50), -0.15185632978888303f64),
        ((50, 100, 100), -0.33800073097192557f64),
        ((50, 200, -100), -0.1262053025774186f64),
        ((50, 200, -50), -0.18678102576752423f64),
        ((50, 200, 0), -0.04298915312937759f64),
        ((50, 200, 50), -0.35937135281916827f64),
        ((50, 200, 100), -0.09675303361528802f64),
        ((100, -200, -100), 0.016944564179341898f64),
        ((100, -200, -50), -0.21449082979744338f64),
        ((100, -200, 0), -0.4864973070953402f64),
        ((100, -200, 50), -0.12082732785556784f64),
        ((100, -200, 100), 0.15105512670391716f64),
        ((100, -100, -100), -0.42014790810555663f64),
        ((100, -100, -50), 0.25043337086794476f64),
        ((100, -100, 0), 0.4836407742236192f64),
        ((100, -100, 50), -0.09839641176754102f64),
        ((100, -100, 100), -0.7118185993515945f64),
        ((100, 0, -100), -0.452981644351176f64),
        ((100, 0, -50), 0.3195442816621561f64),
        ((100, 0, 0), -0.316964588789998f64),
        ((100, 0, 50), -0.09085595379884051f64),
        ((100, 0, 100), -0.18535799255754892f64),
        ((100, 100, -100), 0.21432773343101275f64),
        ((100, 100, -50), -0.31712332334064697f64),
        ((100, 100, 0), -0.2560240287841878f64),
        ((100, 100, 50), -0.09580536400123087f64),
        ((100, 100, 100), -0.0992129190886302f64),
        ((100, 200, -100), 0.41460868389055017f64),
        ((100, 200, -50), 0.4415181826498342f64),
        ((100, 200, 0), 0.1205037616719153f64),
        ((100, 200, 50), -0.7214410961887224f64),
        ((100, 200, 100), 0.3867496985743827f64),
    ];
    for ((x, y, z), value) in values {
        let pos = TestNoisePos { x, y, z };
        assert_eq!(sample_router_function!(vein_gap, pos), value);
    }
}

#[test]
fn test_config_final_density() {
    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/final_density_dump_7_4.json");

    let function_ast = NOISE_ROUTER_ASTS.overworld().final_density();
    let proto_stack = build_proto_stack!(function_ast);
    let mut stack = build_function_stack!(proto_stack);
    let mut function = build_function!(stack);

    // This is a lot of data it iter over, one two skip a few done
    for (x, y, z, sample) in expected_data.into_iter().step_by(5) {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[derive(Deserialize)]
struct DensityFunctionReprs {
    #[serde(rename = "overworld/base_3d_noise")]
    base_3d_noise: DensityFunctionRepr,
    #[serde(rename = "overworld/caves/spaghetti_2d_thickness_modulator")]
    spaghetti_2d_thickness: DensityFunctionRepr,
    #[serde(rename = "overworld/caves/pillars")]
    cave_pillars: DensityFunctionRepr,
    #[serde(rename = "overworld/caves/noodle")]
    cave_noodle: DensityFunctionRepr,
    #[serde(rename = "overworld/caves/spaghetti_roughness_function")]
    spaghetti_roughness: DensityFunctionRepr,
    #[serde(rename = "overworld/caves/entrances")]
    cave_entrances: DensityFunctionRepr,
    #[serde(rename = "overworld/caves/spaghetti_2d")]
    spaghetti_2d: DensityFunctionRepr,
    #[serde(rename = "overworld/offset")]
    offset: DensityFunctionRepr,
    #[serde(rename = "overworld/depth")]
    depth: DensityFunctionRepr,
    #[serde(rename = "overworld/factor")]
    factor: DensityFunctionRepr,
    #[serde(rename = "overworld/sloped_cheese")]
    sloped_cheese: DensityFunctionRepr,
}

macro_rules! read_data_from_file_json5 {
    ($path:expr) => {
        serde_json5::from_str(
            &fs::read_to_string(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .unwrap()
                    .join(file!())
                    .parent()
                    .unwrap()
                    .join($path),
            )
            .expect("no data file"),
        )
        .expect("failed to decode data")
    };
}

static DENSITY_FUNCTION_REPRS: LazyLock<DensityFunctionReprs> =
    LazyLock::new(|| read_data_from_file_json5!("../../../../assets/density_function_tests.json"));

#[test]
fn test_base_sloped_cheese() {
    let proto_stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.sloped_cheese);
    let mut stack = build_function_stack!(proto_stack);
    let mut function = build_function!(stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_sloped_cheese_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_factor() {
    let proto_stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.factor);
    let mut stack = build_function_stack!(proto_stack);
    let mut function = build_function!(stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_factor_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_depth() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.depth);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_depth_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_offset() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.offset);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_offset_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_cave_entrances() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.cave_entrances);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_cave_entrances_overworld_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_3d_noise() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.base_3d_noise);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_3d_overworld_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_spahetti_roughness() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.spaghetti_roughness);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> = read_data_from_file!(
        "../../../../assets/converted_cave_spaghetti_rough_overworld_7_4.json"
    );
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_cave_noodle() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.cave_noodle);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_cave_noodle_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_cave_pillars() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.cave_pillars);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_cave_pillar_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}

#[test]
fn test_base_spaghetti_2d_thickness() {
    let stack = build_proto_stack!(&DENSITY_FUNCTION_REPRS.spaghetti_2d_thickness);
    let mut function_stack = build_function_stack!(stack);
    let mut function = build_function!(function_stack);

    let expected_data: Vec<(i32, i32, i32, f64)> =
        read_data_from_file!("../../../../assets/converted_cave_spaghetti_2d_thicc_7_4.json");
    for (x, y, z, sample) in expected_data {
        let pos = TestNoisePos { x, y, z };
        assert_eq_delta!(
            function.sample(&pos, &TEST_SAMPLE_OPTIONS),
            sample,
            f64::EPSILON
        );
    }
}
