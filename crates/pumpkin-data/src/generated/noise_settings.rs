/* This file is generated. Do not edit manually. */
use crate::BlockState;
use crate::biome::ParameterPoint;
use crate::dimension::Dimension;
pub struct NoiseSettings {
    pub aquifers_enabled: bool,
    pub ore_veins_enabled: bool,
    pub legacy_random_source: bool,
    pub sea_level: i32,
    pub default_fluid: &'static BlockState,
    pub shape: GenerationShapeConfig,
    pub default_block: &'static BlockState,
    pub spawn_target: &'static [ParameterPoint],
}
pub type GenerationSettings = NoiseSettings;
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
    #[must_use]
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
impl NoiseSettings {
    pub const AMPLIFIED: NoiseSettings = NoiseSettings {
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
        sea_level: 63i32,
        default_fluid: crate::Block::WATER.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 384u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        default_block: crate::Block::STONE.default_state,
        spawn_target: &[
            crate::biome::ParameterPoint {
                temperature: crate::biome::Parameter::new(-10000i64, 10000i64),
                humidity: crate::biome::Parameter::new(-10000i64, 10000i64),
                continentalness: crate::biome::Parameter::new(-1100i64, 10000i64),
                erosion: crate::biome::Parameter::new(-10000i64, 10000i64),
                depth: crate::biome::Parameter::new(0i64, 0i64),
                weirdness: crate::biome::Parameter::new(-10000i64, -1600i64),
                offset: 0i64,
            },
            crate::biome::ParameterPoint {
                temperature: crate::biome::Parameter::new(-10000i64, 10000i64),
                humidity: crate::biome::Parameter::new(-10000i64, 10000i64),
                continentalness: crate::biome::Parameter::new(-1100i64, 10000i64),
                erosion: crate::biome::Parameter::new(-10000i64, 10000i64),
                depth: crate::biome::Parameter::new(0i64, 0i64),
                weirdness: crate::biome::Parameter::new(1600i64, 10000i64),
                offset: 0i64,
            },
        ],
    };
    pub const CAVES: NoiseSettings = NoiseSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: 32i32,
        default_fluid: crate::Block::WATER.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 192u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        default_block: crate::Block::STONE.default_state,
        spawn_target: &[],
    };
    pub const END: NoiseSettings = NoiseSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: 0i32,
        default_fluid: crate::Block::AIR.default_state,
        shape: GenerationShapeConfig {
            min_y: 0i8,
            height: 128u16,
            size_horizontal: 2u8,
            size_vertical: 1u8,
        },
        default_block: crate::Block::END_STONE.default_state,
        spawn_target: &[],
    };
    pub const FLOATING_ISLANDS: NoiseSettings = NoiseSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: -64i32,
        default_fluid: crate::Block::WATER.default_state,
        shape: GenerationShapeConfig {
            min_y: 0i8,
            height: 256u16,
            size_horizontal: 2u8,
            size_vertical: 1u8,
        },
        default_block: crate::Block::STONE.default_state,
        spawn_target: &[],
    };
    pub const LARGE_BIOMES: NoiseSettings = NoiseSettings {
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
        sea_level: 63i32,
        default_fluid: crate::Block::WATER.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 384u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        default_block: crate::Block::STONE.default_state,
        spawn_target: &[
            crate::biome::ParameterPoint {
                temperature: crate::biome::Parameter::new(-10000i64, 10000i64),
                humidity: crate::biome::Parameter::new(-10000i64, 10000i64),
                continentalness: crate::biome::Parameter::new(-1100i64, 10000i64),
                erosion: crate::biome::Parameter::new(-10000i64, 10000i64),
                depth: crate::biome::Parameter::new(0i64, 0i64),
                weirdness: crate::biome::Parameter::new(-10000i64, -1600i64),
                offset: 0i64,
            },
            crate::biome::ParameterPoint {
                temperature: crate::biome::Parameter::new(-10000i64, 10000i64),
                humidity: crate::biome::Parameter::new(-10000i64, 10000i64),
                continentalness: crate::biome::Parameter::new(-1100i64, 10000i64),
                erosion: crate::biome::Parameter::new(-10000i64, 10000i64),
                depth: crate::biome::Parameter::new(0i64, 0i64),
                weirdness: crate::biome::Parameter::new(1600i64, 10000i64),
                offset: 0i64,
            },
        ],
    };
    pub const NETHER: NoiseSettings = NoiseSettings {
        aquifers_enabled: false,
        ore_veins_enabled: false,
        legacy_random_source: true,
        sea_level: 32i32,
        default_fluid: crate::Block::LAVA.default_state,
        shape: GenerationShapeConfig {
            min_y: 0i8,
            height: 128u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        default_block: crate::Block::NETHERRACK.default_state,
        spawn_target: &[],
    };
    pub const OVERWORLD: NoiseSettings = NoiseSettings {
        aquifers_enabled: true,
        ore_veins_enabled: true,
        legacy_random_source: false,
        sea_level: 63i32,
        default_fluid: crate::Block::WATER.default_state,
        shape: GenerationShapeConfig {
            min_y: -64i8,
            height: 384u16,
            size_horizontal: 1u8,
            size_vertical: 2u8,
        },
        default_block: crate::Block::STONE.default_state,
        spawn_target: &[
            crate::biome::ParameterPoint {
                temperature: crate::biome::Parameter::new(-10000i64, 10000i64),
                humidity: crate::biome::Parameter::new(-10000i64, 10000i64),
                continentalness: crate::biome::Parameter::new(-1100i64, 10000i64),
                erosion: crate::biome::Parameter::new(-10000i64, 10000i64),
                depth: crate::biome::Parameter::new(0i64, 0i64),
                weirdness: crate::biome::Parameter::new(-10000i64, -1600i64),
                offset: 0i64,
            },
            crate::biome::ParameterPoint {
                temperature: crate::biome::Parameter::new(-10000i64, 10000i64),
                humidity: crate::biome::Parameter::new(-10000i64, 10000i64),
                continentalness: crate::biome::Parameter::new(-1100i64, 10000i64),
                erosion: crate::biome::Parameter::new(-10000i64, 10000i64),
                depth: crate::biome::Parameter::new(0i64, 0i64),
                weirdness: crate::biome::Parameter::new(1600i64, 10000i64),
                offset: 0i64,
            },
        ],
    };
    #[must_use]
    pub fn from_dimension(dimension: &Dimension) -> &'static Self {
        if dimension == &Dimension::OVERWORLD {
            &Self::OVERWORLD
        } else if dimension == &Dimension::THE_NETHER {
            &Self::NETHER
        } else {
            &Self::END
        }
    }
}
