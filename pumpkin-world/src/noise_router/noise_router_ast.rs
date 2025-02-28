use derive_getters::Getters;
use serde::Deserialize;

use super::density_function_ast::DensityFunctionRepr;

#[allow(dead_code)]
#[derive(Deserialize, Getters)]
pub struct NoiseRouterReprs {
    pub(crate) overworld: NoiseRouterRepr,
    #[serde(rename(deserialize = "large_biomes"))]
    pub(crate) overworld_large_biomes: NoiseRouterRepr,
    #[serde(rename(deserialize = "amplified"))]
    pub(crate) overworld_amplified: NoiseRouterRepr,
    pub(crate) nether: NoiseRouterRepr,
    pub(crate) end: NoiseRouterRepr,
    #[serde(rename(deserialize = "floating_islands"))]
    pub(crate) end_islands: NoiseRouterRepr,
}

#[derive(Deserialize, Getters)]
pub struct NoiseRouterRepr {
    #[serde(rename(deserialize = "barrierNoise"))]
    barrier_noise: DensityFunctionRepr,
    #[serde(rename(deserialize = "fluidLevelFloodednessNoise"))]
    fluid_level_floodedness_noise: DensityFunctionRepr,
    #[serde(rename(deserialize = "fluidLevelSpreadNoise"))]
    fluid_level_spread_noise: DensityFunctionRepr,
    #[serde(rename(deserialize = "lavaNoise"))]
    lava_noise: DensityFunctionRepr,
    temperature: DensityFunctionRepr,
    vegetation: DensityFunctionRepr,
    continents: DensityFunctionRepr,
    erosion: DensityFunctionRepr,
    depth: DensityFunctionRepr,
    ridges: DensityFunctionRepr,
    #[serde(rename(deserialize = "initialDensityWithoutJaggedness"))]
    initial_density_without_jaggedness: DensityFunctionRepr,
    #[serde(rename(deserialize = "finalDensity"))]
    pub(crate) final_density: DensityFunctionRepr,
    #[serde(rename(deserialize = "veinToggle"))]
    vein_toggle: DensityFunctionRepr,
    #[serde(rename(deserialize = "veinRidged"))]
    vein_ridged: DensityFunctionRepr,
    #[serde(rename(deserialize = "veinGap"))]
    vein_gap: DensityFunctionRepr,
}
