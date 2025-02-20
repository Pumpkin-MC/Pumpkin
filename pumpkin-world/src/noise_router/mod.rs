use std::sync::LazyLock;

use density_function_ast::{
    BinaryData, BinaryOperation, DensityFunctionRepr, HashableF64, WrapperType,
};
use noise_router_ast::NoiseRouterReprs;

pub mod density_function_ast;
pub mod noise_router_ast;

macro_rules! fix_final_density {
    ($router:expr) => {{
        $router.final_density = DensityFunctionRepr::Wrapper {
            input: Box::new(DensityFunctionRepr::Binary {
                argument1: Box::new($router.final_density),
                argument2: Box::new(DensityFunctionRepr::Beardifier),
                data: BinaryData {
                    operation: BinaryOperation::Add,
                    max_value: HashableF64(f64::INFINITY),
                    min_value: HashableF64(f64::NEG_INFINITY),
                },
            }),
            wrapper: WrapperType::CellCache,
        };
    }};
}

pub static NOISE_ROUTER_ASTS: LazyLock<NoiseRouterReprs> = LazyLock::new(|| {
    // JSON5 is needed because of NaN, Inf, and -Inf
    let mut reprs: NoiseRouterReprs =
        serde_json5::from_str(include_str!("../../../assets/density_function.json"))
            .expect("could not deserialize density_function.json");

    // The `final_density` function is mutated at runtime for the aquifer generator.
    fix_final_density!(reprs.overworld);
    fix_final_density!(reprs.overworld_amplified);
    fix_final_density!(reprs.overworld_large_biomes);
    fix_final_density!(reprs.nether);

    reprs
});
