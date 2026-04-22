/* This file is generated. Do not edit manually. */
use crate::BlockState;
use crate::biome::Biome;
use crate::chunk::DoublePerlinNoiseParameters;
use crate::dimension::Dimension;
use pumpkin_util::math::vertical_surface_type::VerticalSurfaceType;
use pumpkin_util::y_offset::YOffset;
pub struct GenerationSettings {
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub default_fluid: &'static BlockState,
    pub default_block: &'static BlockState,
    pub shape: GenerationShapeConfig,
    #[doc = r" Flat compiled surface rule — no recursion at runtime."]
    pub surface_rule: CompiledSurfaceRule,
}
pub struct GenerationShapeConfig {
    pub min_y: i8,
    pub height: u16,
    pub size_horizontal: u8,
    pub size_vertical: u8,
}
impl GenerationShapeConfig {
    #[inline]
    #[must_use]
    pub const fn vertical_cell_block_count(&self) -> u8 {
        self.size_vertical << 2
    }
    #[inline]
    #[must_use]
    pub const fn horizontal_cell_block_count(&self) -> u8 {
        self.size_horizontal << 2
    }
    #[must_use]
    pub const fn max_y(&self) -> u16 {
        if self.min_y >= 0 {
            self.height + self.min_y as u16
        } else {
            (self.height as i32 + self.min_y as i32) as u16
        }
    }
    pub fn trim_height(&self, bottom_y: i8, top_y: u16) -> Self {
        let new_min = self.min_y.max(bottom_y);
        let this_top = if self.min_y >= 0 {
            self.height + self.min_y as u16
        } else {
            self.height - self.min_y.unsigned_abs() as u16
        };
        let new_top = this_top.min(top_y);
        let new_height = if new_min >= 0 {
            new_top - new_min as u16
        } else {
            new_top + new_min.unsigned_abs() as u16
        };
        Self {
            min_y: new_min,
            height: new_height,
            size_horizontal: self.size_horizontal,
            size_vertical: self.size_vertical,
        }
    }
}
#[doc = r" A compiled, flat representation of a surface rule tree."]
#[doc = r" Evaluated by a simple index-advancing loop — no recursion."]
pub struct CompiledSurfaceRule {
    pub instructions: &'static [SurfaceInstruction],
}
#[doc = r" One instruction in the flat surface rule bytecode."]
#[doc = r""]
#[doc = r" Test instructions carry a `skip: u16` field. On failure the evaluator"]
#[doc = r" advances the program counter by `skip + 1` (past the body). On success"]
#[doc = r" it advances by 1 (into the body). Terminals end evaluation immediately."]
pub enum SurfaceInstruction {
    PlaceBlock {
        state: &'static BlockState,
    },
    PlaceBadlands,
    TestBiome {
        biome_is: &'static [&'static Biome],
        skip: u16,
    },
    #[doc = r" Noise >= min  (the f64::MAX upper-bound fast path)"]
    TestNoiseAbove {
        noise: DoublePerlinNoiseParameters,
        min: f64,
        skip: u16,
    },
    #[doc = r" min <= noise <= max"]
    TestNoiseRange {
        noise: DoublePerlinNoiseParameters,
        min: f64,
        max: f64,
        skip: u16,
    },
    TestVerticalGradient {
        random_lo: u64,
        random_hi: u64,
        true_at_and_below: YOffset,
        false_at_and_above: YOffset,
        skip: u16,
    },
    TestYAbove {
        anchor: YOffset,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
        skip: u16,
    },
    TestWater {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
        skip: u16,
    },
    TestStoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        surface_type: VerticalSurfaceType,
        skip: u16,
    },
    TestAbovePreliminarySurface {
        skip: u16,
    },
    TestHole {
        skip: u16,
    },
    TestSteep {
        skip: u16,
    },
    TestTemperature {
        skip: u16,
    },
    #[doc = r" Inverts a single inner condition."]
    #[doc = r" The inner condition is stored inline — it must not itself contain"]
    #[doc = r" a body (i.e. it always has inner skip = 0)."]
    TestNot {
        inner: &'static SurfaceInstruction,
        skip: u16,
    },
}
#[doc = r" Caches noise samples for one (x, z) column."]
#[doc = r" Call `invalidate(x, z)` once per column; `get` then returns the cached"]
#[doc = r" value on subsequent calls for the same noise parameter."]
pub struct ColumnNoiseCache {
    values: [f64; DoublePerlinNoiseParameters::COUNT],
    valid: [bool; DoublePerlinNoiseParameters::COUNT],
    col_x: i32,
    col_z: i32,
}
impl ColumnNoiseCache {
    pub const fn new() -> Self {
        Self {
            values: [0.0; DoublePerlinNoiseParameters::COUNT],
            valid: [false; DoublePerlinNoiseParameters::COUNT],
            col_x: i32::MIN,
            col_z: i32::MIN,
        }
    }
    #[doc = r" Must be called at the start of each new (x, z) column."]
    #[inline]
    pub fn invalidate(&mut self, x: i32, z: i32) {
        if self.col_x != x || self.col_z != z {
            self.valid = [false; DoublePerlinNoiseParameters::COUNT];
            self.col_x = x;
            self.col_z = z;
        }
    }
    #[doc = r" Returns the cached noise value, sampling it on first access."]
    #[inline]
    pub fn get(
        &mut self,
        noise: &DoublePerlinNoiseParameters,
        sample: impl FnOnce() -> f64,
    ) -> f64 {
        let idx = noise.id;
        if !self.valid[idx] {
            self.values[idx] = sample();
            self.valid[idx] = true;
        }
        self.values[idx]
    }
}
impl Default for ColumnNoiseCache {
    fn default() -> Self {
        Self::new()
    }
}
impl GenerationSettings {
    pub const AMPLIFIED: GenerationSettings = GenerationSettings {
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
        sea_level: 63i32,
        default_fluid: crate::Block::WATER.default_state,
        default_block: crate::Block::STONE.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 384u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 13544455532117611141u64,
                    random_hi: 14185350335435586452u64,
                    true_at_and_below: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 0i8,
                    }),
                    false_at_and_above: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 5i8,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestAbovePreliminarySurface { skip: 234u16 },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WOODED_BADLANDS],
                    skip: 10u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 97i16 }),
                    surface_depth_multiplier: 2i32,
                    add_stone_depth: false,
                    skip: 9u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 62i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 60i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::BADLANDS,
                        &crate::biome::Biome::ERODED_BADLANDS,
                        &crate::biome::Biome::WOODED_BADLANDS,
                    ],
                    skip: 30u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 256i16,
                    }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 74i16 }),
                    surface_depth_multiplier: 1i32,
                    add_stone_depth: true,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SAND.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 74i16,
                        }),
                        surface_depth_multiplier: 1i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 93u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 92u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 6u16,
                },
                SurfaceInstruction::TestHole { skip: 5u16 },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::AIR.default_state,
                },
                SurfaceInstruction::TestTemperature { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: 0f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: 0f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 4u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.06060606060606061f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 13u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::OLD_GROWTH_PINE_TAIGA,
                        &crate::biome::Biome::OLD_GROWTH_SPRUCE_TAIGA,
                    ],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.11515151515151514f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PODZOL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::ICE_SPIKES],
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MUSHROOM_FIELDS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MYCELIUM.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 76u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 3u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestHole { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 65u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: -0.5f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: -0.0625f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 11u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 6i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 30i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 9u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_PEAKS,
                        &crate::biome::Biome::JAGGED_PEAKS,
                    ],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::LUKEWARM_OCEAN,
                        &crate::biome::Biome::DEEP_LUKEWARM_OCEAN,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 10411719568726253007u64,
                    random_hi: 14964796469053385315u64,
                    true_at_and_below: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 0i16,
                    }),
                    false_at_and_above: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 8i16,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DEEPSLATE.default_state,
                },
            ],
        },
    };
    pub const CAVES: GenerationSettings = GenerationSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: 32i32,
        default_fluid: crate::Block::WATER.default_state,
        default_block: crate::Block::STONE.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 192u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestVerticalGradient {
                        random_lo: 10285458612719284684u64,
                        random_hi: 14520959502364845764u64,
                        true_at_and_below: YOffset::BelowTop(pumpkin_util::y_offset::BelowTop {
                            below_top: 5i8,
                        }),
                        false_at_and_above: YOffset::BelowTop(pumpkin_util::y_offset::BelowTop {
                            below_top: 0i8,
                        }),
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 13544455532117611141u64,
                    random_hi: 14185350335435586452u64,
                    true_at_and_below: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 0i8,
                    }),
                    false_at_and_above: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 5i8,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WOODED_BADLANDS],
                    skip: 10u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 97i16 }),
                    surface_depth_multiplier: 2i32,
                    add_stone_depth: false,
                    skip: 9u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 62i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 60i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::BADLANDS,
                        &crate::biome::Biome::ERODED_BADLANDS,
                        &crate::biome::Biome::WOODED_BADLANDS,
                    ],
                    skip: 30u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 256i16,
                    }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 74i16 }),
                    surface_depth_multiplier: 1i32,
                    add_stone_depth: true,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SAND.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 74i16,
                        }),
                        surface_depth_multiplier: 1i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 93u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 92u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 6u16,
                },
                SurfaceInstruction::TestHole { skip: 5u16 },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::AIR.default_state,
                },
                SurfaceInstruction::TestTemperature { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: 0f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: 0f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 4u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.06060606060606061f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 13u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::OLD_GROWTH_PINE_TAIGA,
                        &crate::biome::Biome::OLD_GROWTH_SPRUCE_TAIGA,
                    ],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.11515151515151514f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PODZOL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::ICE_SPIKES],
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MUSHROOM_FIELDS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MYCELIUM.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 76u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 3u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestHole { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 65u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: -0.5f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: -0.0625f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 11u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 6i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 30i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 9u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_PEAKS,
                        &crate::biome::Biome::JAGGED_PEAKS,
                    ],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::LUKEWARM_OCEAN,
                        &crate::biome::Biome::DEEP_LUKEWARM_OCEAN,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 10411719568726253007u64,
                    random_hi: 14964796469053385315u64,
                    true_at_and_below: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 0i16,
                    }),
                    false_at_and_above: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 8i16,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DEEPSLATE.default_state,
                },
            ],
        },
    };
    pub const END: GenerationSettings = GenerationSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: 0i32,
        default_fluid: crate::Block::AIR.default_state,
        default_block: crate::Block::END_STONE.default_state,
        shape: GenerationShapeConfig {
            min_y: 0i8,
            height: 128u16,
            size_horizontal: 2u8,
            size_vertical: 1u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[SurfaceInstruction::PlaceBlock {
                state: crate::Block::END_STONE.default_state,
            }],
        },
    };
    pub const FLOATING_ISLANDS: GenerationSettings = GenerationSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: -64i32,
        default_fluid: crate::Block::WATER.default_state,
        default_block: crate::Block::STONE.default_state,
        shape: GenerationShapeConfig {
            min_y: 0i8,
            height: 256u16,
            size_horizontal: 2u8,
            size_vertical: 1u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WOODED_BADLANDS],
                    skip: 10u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 97i16 }),
                    surface_depth_multiplier: 2i32,
                    add_stone_depth: false,
                    skip: 9u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 62i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 60i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::BADLANDS,
                        &crate::biome::Biome::ERODED_BADLANDS,
                        &crate::biome::Biome::WOODED_BADLANDS,
                    ],
                    skip: 30u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 256i16,
                    }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 74i16 }),
                    surface_depth_multiplier: 1i32,
                    add_stone_depth: true,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SAND.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 74i16,
                        }),
                        surface_depth_multiplier: 1i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 93u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 92u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 6u16,
                },
                SurfaceInstruction::TestHole { skip: 5u16 },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::AIR.default_state,
                },
                SurfaceInstruction::TestTemperature { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: 0f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: 0f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 4u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.06060606060606061f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 13u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::OLD_GROWTH_PINE_TAIGA,
                        &crate::biome::Biome::OLD_GROWTH_SPRUCE_TAIGA,
                    ],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.11515151515151514f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PODZOL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::ICE_SPIKES],
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MUSHROOM_FIELDS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MYCELIUM.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 76u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 3u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestHole { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 65u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: -0.5f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: -0.0625f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 11u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 6i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 30i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 9u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_PEAKS,
                        &crate::biome::Biome::JAGGED_PEAKS,
                    ],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::LUKEWARM_OCEAN,
                        &crate::biome::Biome::DEEP_LUKEWARM_OCEAN,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 10411719568726253007u64,
                    random_hi: 14964796469053385315u64,
                    true_at_and_below: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 0i16,
                    }),
                    false_at_and_above: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 8i16,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DEEPSLATE.default_state,
                },
            ],
        },
    };
    pub const LARGE_BIOMES: GenerationSettings = GenerationSettings {
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
        sea_level: 63i32,
        default_fluid: crate::Block::WATER.default_state,
        default_block: crate::Block::STONE.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 384u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 13544455532117611141u64,
                    random_hi: 14185350335435586452u64,
                    true_at_and_below: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 0i8,
                    }),
                    false_at_and_above: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 5i8,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestAbovePreliminarySurface { skip: 234u16 },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WOODED_BADLANDS],
                    skip: 10u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 97i16 }),
                    surface_depth_multiplier: 2i32,
                    add_stone_depth: false,
                    skip: 9u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 62i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 60i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::BADLANDS,
                        &crate::biome::Biome::ERODED_BADLANDS,
                        &crate::biome::Biome::WOODED_BADLANDS,
                    ],
                    skip: 30u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 256i16,
                    }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 74i16 }),
                    surface_depth_multiplier: 1i32,
                    add_stone_depth: true,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SAND.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 74i16,
                        }),
                        surface_depth_multiplier: 1i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 93u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 92u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 6u16,
                },
                SurfaceInstruction::TestHole { skip: 5u16 },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::AIR.default_state,
                },
                SurfaceInstruction::TestTemperature { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: 0f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: 0f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 4u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.06060606060606061f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 13u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::OLD_GROWTH_PINE_TAIGA,
                        &crate::biome::Biome::OLD_GROWTH_SPRUCE_TAIGA,
                    ],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.11515151515151514f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PODZOL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::ICE_SPIKES],
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MUSHROOM_FIELDS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MYCELIUM.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 76u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 3u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestHole { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 65u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: -0.5f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: -0.0625f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 11u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 6i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 30i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 9u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_PEAKS,
                        &crate::biome::Biome::JAGGED_PEAKS,
                    ],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::LUKEWARM_OCEAN,
                        &crate::biome::Biome::DEEP_LUKEWARM_OCEAN,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 10411719568726253007u64,
                    random_hi: 14964796469053385315u64,
                    true_at_and_below: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 0i16,
                    }),
                    false_at_and_above: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 8i16,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DEEPSLATE.default_state,
                },
            ],
        },
    };
    pub const NETHER: GenerationSettings = GenerationSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: 32i32,
        default_fluid: crate::Block::LAVA.default_state,
        default_block: crate::Block::NETHERRACK.default_state,
        shape: GenerationShapeConfig {
            min_y: 0i8,
            height: 128u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 13544455532117611141u64,
                    random_hi: 14185350335435586452u64,
                    true_at_and_below: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 0i8,
                    }),
                    false_at_and_above: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 5i8,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestVerticalGradient {
                        random_lo: 10285458612719284684u64,
                        random_hi: 14520959502364845764u64,
                        true_at_and_below: YOffset::BelowTop(pumpkin_util::y_offset::BelowTop {
                            below_top: 5i8,
                        }),
                        false_at_and_above: YOffset::BelowTop(pumpkin_util::y_offset::BelowTop {
                            below_top: 0i8,
                        }),
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::BelowTop(pumpkin_util::y_offset::BelowTop { below_top: 5i8 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::NETHERRACK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::BASALT_DELTAS],
                    skip: 10u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BASALT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::PATCH,
                    min: -0.012f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 30i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: true,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 35i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::NETHER_STATE_SELECTOR,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BASALT.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BLACKSTONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SOUL_SAND_VALLEY],
                    skip: 12u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::NETHER_STATE_SELECTOR,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SOUL_SAND.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SOUL_SOIL.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::PATCH,
                    min: -0.012f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 30i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: true,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 35i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::NETHER_STATE_SELECTOR,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SOUL_SAND.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SOUL_SOIL.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 15u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 32i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestHole { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::LAVA.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WARPED_FOREST],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestNoiseAbove {
                        noise: DoublePerlinNoiseParameters::NETHERRACK,
                        min: 0.54f64,
                        skip: 0u16,
                    },
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 31i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::NETHER_WART,
                    min: 1.17f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WARPED_WART_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WARPED_NYLIUM.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::CRIMSON_FOREST],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestNoiseAbove {
                        noise: DoublePerlinNoiseParameters::NETHERRACK,
                        min: 0.54f64,
                        skip: 0u16,
                    },
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 31i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::NETHER_WART,
                    min: 1.17f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::NETHER_WART_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CRIMSON_NYLIUM.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::NETHER_WASTES],
                    skip: 15u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 6u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SOUL_SAND_LAYER,
                    min: -0.012f64,
                    skip: 5u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 3u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 30i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: true,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 35i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SOUL_SAND.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::NETHERRACK.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 7u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 31i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 6u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 35i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::GRAVEL_LAYER,
                    min: -0.012f64,
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 32i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::NETHERRACK.default_state,
                },
            ],
        },
    };
    pub const OVERWORLD: GenerationSettings = GenerationSettings {
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
        sea_level: 63i32,
        default_fluid: crate::Block::WATER.default_state,
        default_block: crate::Block::STONE.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 384u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        surface_rule: CompiledSurfaceRule {
            instructions: &[
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 13544455532117611141u64,
                    random_hi: 14185350335435586452u64,
                    true_at_and_below: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 0i8,
                    }),
                    false_at_and_above: YOffset::AboveBottom(pumpkin_util::y_offset::AboveBottom {
                        above_bottom: 5i8,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::BEDROCK.default_state,
                },
                SurfaceInstruction::TestAbovePreliminarySurface { skip: 234u16 },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WOODED_BADLANDS],
                    skip: 10u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 97i16 }),
                    surface_depth_multiplier: 2i32,
                    add_stone_depth: false,
                    skip: 9u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 62i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 60i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 63i16,
                        }),
                        surface_depth_multiplier: 0i32,
                        add_stone_depth: false,
                        skip: 0u16,
                    },
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE_SWAMP,
                    min: 0f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::BADLANDS,
                        &crate::biome::Biome::ERODED_BADLANDS,
                        &crate::biome::Biome::WOODED_BADLANDS,
                    ],
                    skip: 30u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 21u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 256i16,
                    }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 74i16 }),
                    surface_depth_multiplier: 1i32,
                    add_stone_depth: true,
                    skip: 7u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.909f64,
                    max: -0.5454f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.1818f64,
                    max: 0.1818f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.5454f64,
                    max: 0.909f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::RED_SAND.default_state,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestHole { skip: 0u16 },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 4u16,
                },
                SurfaceInstruction::TestYAbove {
                    anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute { absolute: 63i16 }),
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 2u16,
                },
                SurfaceInstruction::TestNot {
                    inner: &SurfaceInstruction::TestYAbove {
                        anchor: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                            absolute: 74i16,
                        }),
                        surface_depth_multiplier: 1i32,
                        add_stone_depth: true,
                        skip: 0u16,
                    },
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ORANGE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::PlaceBadlands,
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WHITE_TERRACOTTA.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 93u16,
                },
                SurfaceInstruction::TestWater {
                    offset: -1i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 92u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 6u16,
                },
                SurfaceInstruction::TestHole { skip: 5u16 },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::AIR.default_state,
                },
                SurfaceInstruction::TestTemperature { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: 0f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: 0f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 4u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.35f64,
                    max: 0.6f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.06060606060606061f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 13u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::OLD_GROWTH_PINE_TAIGA,
                        &crate::biome::Biome::OLD_GROWTH_SPRUCE_TAIGA,
                    ],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::COARSE_DIRT.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.11515151515151514f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PODZOL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::ICE_SPIKES],
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MUSHROOM_FIELDS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MYCELIUM.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRASS_BLOCK.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: -6i32,
                    surface_depth_multiplier: -1i32,
                    add_stone_depth: true,
                    skip: 76u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 3u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_OCEAN,
                        &crate::biome::Biome::DEEP_FROZEN_OCEAN,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestHole { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::WATER.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 65u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::FROZEN_PEAKS],
                    skip: 8u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::PACKED_ICE,
                    min: -0.5f64,
                    max: 0.2f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::PACKED_ICE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::ICE,
                    min: -0.0625f64,
                    max: 0.025f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::ICE.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::SNOWY_SLOPES],
                    skip: 7u16,
                },
                SurfaceInstruction::TestSteep { skip: 1u16 },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SNOW_BLOCK.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::JAGGED_PEAKS],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::GROVE],
                    skip: 4u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::POWDER_SNOW,
                    min: 0.45f64,
                    max: 0.58f64,
                    skip: 2u16,
                },
                SurfaceInstruction::TestWater {
                    offset: 0i32,
                    surface_depth_multiplier: 0i32,
                    add_stone_depth: false,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::POWDER_SNOW.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_PEAKS],
                    skip: 3u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::CALCITE,
                    min: -0.0125f64,
                    max: 0.0125f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::CALCITE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::STONY_SHORE],
                    skip: 5u16,
                },
                SurfaceInstruction::TestNoiseRange {
                    noise: DoublePerlinNoiseParameters::GRAVEL,
                    min: -0.05f64,
                    max: 0.05f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_HILLS],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DRIPSTONE_CAVES],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_SAVANNA],
                    skip: 2u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.21212121212121213f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::WINDSWEPT_GRAVELLY_HILLS],
                    skip: 11u16,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.24242424242424243f64,
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: 0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestNoiseAbove {
                    noise: DoublePerlinNoiseParameters::SURFACE,
                    min: -0.12121212121212122f64,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::MANGROVE_SWAMP],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::MUD.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DIRT.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::BEACH,
                        &crate::biome::Biome::SNOWY_BEACH,
                    ],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 6i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[&crate::biome::Biome::DESERT],
                    skip: 2u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: true,
                    secondary_depth_range: 30i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Floor,
                    skip: 9u16,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::FROZEN_PEAKS,
                        &crate::biome::Biome::JAGGED_PEAKS,
                    ],
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::TestBiome {
                    biome_is: &[
                        &crate::biome::Biome::WARM_OCEAN,
                        &crate::biome::Biome::LUKEWARM_OCEAN,
                        &crate::biome::Biome::DEEP_LUKEWARM_OCEAN,
                    ],
                    skip: 3u16,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SANDSTONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::SAND.default_state,
                },
                SurfaceInstruction::TestStoneDepth {
                    offset: 0i32,
                    add_surface_depth: false,
                    secondary_depth_range: 0i32,
                    surface_type: VerticalSurfaceType::Ceiling,
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::STONE.default_state,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::GRAVEL.default_state,
                },
                SurfaceInstruction::TestVerticalGradient {
                    random_lo: 10411719568726253007u64,
                    random_hi: 14964796469053385315u64,
                    true_at_and_below: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 0i16,
                    }),
                    false_at_and_above: YOffset::Absolute(pumpkin_util::y_offset::Absolute {
                        absolute: 8i16,
                    }),
                    skip: 1u16,
                },
                SurfaceInstruction::PlaceBlock {
                    state: crate::Block::DEEPSLATE.default_state,
                },
            ],
        },
    };
    pub fn from_dimension(dimension: &Dimension) -> &'static Self {
        match dimension {
            d if d == &Dimension::OVERWORLD => &Self::OVERWORLD,
            d if d == &Dimension::THE_NETHER => &Self::NETHER,
            _ => &Self::END,
        }
    }
}
