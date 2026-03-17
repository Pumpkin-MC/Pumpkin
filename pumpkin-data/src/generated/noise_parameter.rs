/* This file is generated. Do not edit manually. */
pub struct DoublePerlinNoiseParameters {
    pub first_octave: i32,
    pub amplitudes: &'static [f64],
    id: &'static str,
}
impl DoublePerlinNoiseParameters {
    pub const fn new(first_octave: i32, amplitudes: &'static [f64], id: &'static str) -> Self {
        Self {
            first_octave,
            amplitudes,
            id,
        }
    }
    pub const fn id(&self) -> &'static str {
        self.id
    }
    pub fn id_to_parameters(id: &str) -> Option<&DoublePerlinNoiseParameters> {
        Some(match id {
            "aquifer_barrier" => &Self::AQUIFER_BARRIER,
            "aquifer_fluid_level_floodedness" => &Self::AQUIFER_FLUID_LEVEL_FLOODEDNESS,
            "aquifer_fluid_level_spread" => &Self::AQUIFER_FLUID_LEVEL_SPREAD,
            "aquifer_lava" => &Self::AQUIFER_LAVA,
            "badlands_pillar" => &Self::BADLANDS_PILLAR,
            "badlands_pillar_roof" => &Self::BADLANDS_PILLAR_ROOF,
            "badlands_surface" => &Self::BADLANDS_SURFACE,
            "calcite" => &Self::CALCITE,
            "cave_cheese" => &Self::CAVE_CHEESE,
            "cave_entrance" => &Self::CAVE_ENTRANCE,
            "cave_layer" => &Self::CAVE_LAYER,
            "clay_bands_offset" => &Self::CLAY_BANDS_OFFSET,
            "continentalness" => &Self::CONTINENTALNESS,
            "continentalness_large" => &Self::CONTINENTALNESS_LARGE,
            "erosion" => &Self::EROSION,
            "erosion_large" => &Self::EROSION_LARGE,
            "gravel" => &Self::GRAVEL,
            "gravel_layer" => &Self::GRAVEL_LAYER,
            "ice" => &Self::ICE,
            "iceberg_pillar" => &Self::ICEBERG_PILLAR,
            "iceberg_pillar_roof" => &Self::ICEBERG_PILLAR_ROOF,
            "iceberg_surface" => &Self::ICEBERG_SURFACE,
            "jagged" => &Self::JAGGED,
            "nether_state_selector" => &Self::NETHER_STATE_SELECTOR,
            "nether_wart" => &Self::NETHER_WART,
            "netherrack" => &Self::NETHERRACK,
            "noodle" => &Self::NOODLE,
            "noodle_ridge_a" => &Self::NOODLE_RIDGE_A,
            "noodle_ridge_b" => &Self::NOODLE_RIDGE_B,
            "noodle_thickness" => &Self::NOODLE_THICKNESS,
            "offset" => &Self::OFFSET,
            "ore_gap" => &Self::ORE_GAP,
            "ore_vein_a" => &Self::ORE_VEIN_A,
            "ore_vein_b" => &Self::ORE_VEIN_B,
            "ore_veininess" => &Self::ORE_VEININESS,
            "packed_ice" => &Self::PACKED_ICE,
            "patch" => &Self::PATCH,
            "pillar" => &Self::PILLAR,
            "pillar_rareness" => &Self::PILLAR_RARENESS,
            "pillar_thickness" => &Self::PILLAR_THICKNESS,
            "powder_snow" => &Self::POWDER_SNOW,
            "ridge" => &Self::RIDGE,
            "soul_sand_layer" => &Self::SOUL_SAND_LAYER,
            "spaghetti_2d" => &Self::SPAGHETTI_2D,
            "spaghetti_2d_elevation" => &Self::SPAGHETTI_2D_ELEVATION,
            "spaghetti_2d_modulator" => &Self::SPAGHETTI_2D_MODULATOR,
            "spaghetti_2d_thickness" => &Self::SPAGHETTI_2D_THICKNESS,
            "spaghetti_3d_1" => &Self::SPAGHETTI_3D_1,
            "spaghetti_3d_2" => &Self::SPAGHETTI_3D_2,
            "spaghetti_3d_rarity" => &Self::SPAGHETTI_3D_RARITY,
            "spaghetti_3d_thickness" => &Self::SPAGHETTI_3D_THICKNESS,
            "spaghetti_roughness" => &Self::SPAGHETTI_ROUGHNESS,
            "spaghetti_roughness_modulator" => &Self::SPAGHETTI_ROUGHNESS_MODULATOR,
            "surface" => &Self::SURFACE,
            "surface_secondary" => &Self::SURFACE_SECONDARY,
            "surface_swamp" => &Self::SURFACE_SWAMP,
            "temperature" => &Self::TEMPERATURE,
            "temperature_large" => &Self::TEMPERATURE_LARGE,
            "vegetation" => &Self::VEGETATION,
            "vegetation_large" => &Self::VEGETATION_LARGE,
            _ => return None,
        })
    }
    pub const AQUIFER_BARRIER: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3i32, &[1f64], "minecraft:aquifer_barrier");
    pub const AQUIFER_FLUID_LEVEL_FLOODEDNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(
            -7i32,
            &[1f64],
            "minecraft:aquifer_fluid_level_floodedness",
        );
    pub const AQUIFER_FLUID_LEVEL_SPREAD: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-5i32, &[1f64], "minecraft:aquifer_fluid_level_spread");
    pub const AQUIFER_LAVA: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-1i32, &[1f64], "minecraft:aquifer_lava");
    pub const BADLANDS_PILLAR: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -2i32,
        &[1f64, 1f64, 1f64, 1f64],
        "minecraft:badlands_pillar",
    );
    pub const BADLANDS_PILLAR_ROOF: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:badlands_pillar_roof");
    pub const BADLANDS_SURFACE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6i32, &[1f64, 1f64, 1f64], "minecraft:badlands_surface");
    pub const CALCITE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-9i32, &[1f64, 1f64, 1f64, 1f64], "minecraft:calcite");
    pub const CAVE_CHEESE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8i32,
        &[0.5f64, 1f64, 2f64, 1f64, 2f64, 1f64, 0f64, 2f64, 0f64],
        "minecraft:cave_cheese",
    );
    pub const CAVE_ENTRANCE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[0.4f64, 0.5f64, 1f64], "minecraft:cave_entrance");
    pub const CAVE_LAYER: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:cave_layer");
    pub const CLAY_BANDS_OFFSET: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:clay_bands_offset");
    pub const CONTINENTALNESS: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -9i32,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        "minecraft:continentalness",
    );
    pub const CONTINENTALNESS_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -11i32,
        &[1f64, 1f64, 2f64, 2f64, 2f64, 1f64, 1f64, 1f64, 1f64],
        "minecraft:continentalness_large",
    );
    pub const EROSION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -9i32,
        &[1f64, 1f64, 0f64, 1f64, 1f64],
        "minecraft:erosion",
    );
    pub const EROSION_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -11i32,
        &[1f64, 1f64, 0f64, 1f64, 1f64],
        "minecraft:erosion_large",
    );
    pub const GRAVEL: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64, 1f64, 1f64, 1f64], "minecraft:gravel");
    pub const GRAVEL_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8i32,
        &[
            1f64,
            1f64,
            1f64,
            1f64,
            0f64,
            0f64,
            0f64,
            0f64,
            0.013333333333333334f64,
        ],
        "minecraft:gravel_layer",
    );
    pub const ICE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-4i32, &[1f64, 1f64, 1f64, 1f64], "minecraft:ice");
    pub const ICEBERG_PILLAR: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -6i32,
        &[1f64, 1f64, 1f64, 1f64],
        "minecraft:iceberg_pillar",
    );
    pub const ICEBERG_PILLAR_ROOF: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3i32, &[1f64], "minecraft:iceberg_pillar_roof");
    pub const ICEBERG_SURFACE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6i32, &[1f64, 1f64, 1f64], "minecraft:iceberg_surface");
    pub const JAGGED: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -16i32,
        &[
            1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64, 1f64,
            1f64, 1f64,
        ],
        "minecraft:jagged",
    );
    pub const NETHER_STATE_SELECTOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-4i32, &[1f64], "minecraft:nether_state_selector");
    pub const NETHER_WART: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -3i32,
        &[1f64, 0f64, 0f64, 0.9f64],
        "minecraft:nether_wart",
    );
    pub const NETHERRACK: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -3i32,
        &[1f64, 0f64, 0f64, 0.35f64],
        "minecraft:netherrack",
    );
    pub const NOODLE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:noodle");
    pub const NOODLE_RIDGE_A: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:noodle_ridge_a");
    pub const NOODLE_RIDGE_B: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:noodle_ridge_b");
    pub const NOODLE_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:noodle_thickness");
    pub const OFFSET: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-3i32, &[1f64, 1f64, 1f64, 0f64], "minecraft:offset");
    pub const ORE_GAP: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-5i32, &[1f64], "minecraft:ore_gap");
    pub const ORE_VEIN_A: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:ore_vein_a");
    pub const ORE_VEIN_B: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:ore_vein_b");
    pub const ORE_VEININESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:ore_veininess");
    pub const PACKED_ICE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64, 1f64, 1f64, 1f64], "minecraft:packed_ice");
    pub const PATCH: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -5i32,
        &[1f64, 0f64, 0f64, 0f64, 0f64, 0.013333333333333334f64],
        "minecraft:patch",
    );
    pub const PILLAR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64, 1f64], "minecraft:pillar");
    pub const PILLAR_RARENESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:pillar_rareness");
    pub const PILLAR_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:pillar_thickness");
    pub const POWDER_SNOW: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6i32, &[1f64, 1f64, 1f64, 1f64], "minecraft:powder_snow");
    pub const RIDGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -7i32,
        &[1f64, 2f64, 1f64, 0f64, 0f64, 0f64],
        "minecraft:ridge",
    );
    pub const SOUL_SAND_LAYER: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8i32,
        &[
            1f64,
            1f64,
            1f64,
            1f64,
            0f64,
            0f64,
            0f64,
            0f64,
            0.013333333333333334f64,
        ],
        "minecraft:soul_sand_layer",
    );
    pub const SPAGHETTI_2D: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:spaghetti_2d");
    pub const SPAGHETTI_2D_ELEVATION: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:spaghetti_2d_elevation");
    pub const SPAGHETTI_2D_MODULATOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-11i32, &[1f64], "minecraft:spaghetti_2d_modulator");
    pub const SPAGHETTI_2D_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-11i32, &[1f64], "minecraft:spaghetti_2d_thickness");
    pub const SPAGHETTI_3D_1: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:spaghetti_3d_1");
    pub const SPAGHETTI_3D_2: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-7i32, &[1f64], "minecraft:spaghetti_3d_2");
    pub const SPAGHETTI_3D_RARITY: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-11i32, &[1f64], "minecraft:spaghetti_3d_rarity");
    pub const SPAGHETTI_3D_THICKNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:spaghetti_3d_thickness");
    pub const SPAGHETTI_ROUGHNESS: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-5i32, &[1f64], "minecraft:spaghetti_roughness");
    pub const SPAGHETTI_ROUGHNESS_MODULATOR: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-8i32, &[1f64], "minecraft:spaghetti_roughness_modulator");
    pub const SURFACE: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-6i32, &[1f64, 1f64, 1f64], "minecraft:surface");
    pub const SURFACE_SECONDARY: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -6i32,
        &[1f64, 1f64, 0f64, 1f64],
        "minecraft:surface_secondary",
    );
    pub const SURFACE_SWAMP: DoublePerlinNoiseParameters =
        DoublePerlinNoiseParameters::new(-2i32, &[1f64], "minecraft:surface_swamp");
    pub const TEMPERATURE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -10i32,
        &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
        "minecraft:temperature",
    );
    pub const TEMPERATURE_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -12i32,
        &[1.5f64, 0f64, 1f64, 0f64, 0f64, 0f64],
        "minecraft:temperature_large",
    );
    pub const VEGETATION: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -8i32,
        &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
        "minecraft:vegetation",
    );
    pub const VEGETATION_LARGE: DoublePerlinNoiseParameters = DoublePerlinNoiseParameters::new(
        -10i32,
        &[1f64, 1f64, 0f64, 0f64, 0f64, 0f64],
        "minecraft:vegetation_large",
    );
}
