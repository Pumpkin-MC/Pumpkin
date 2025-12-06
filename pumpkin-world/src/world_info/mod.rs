use std::collections::HashMap;
use std::path::Path;

use crate::CURRENT_MC_VERSION;
use pumpkin_data::game_rules::GameRuleRegistry;
use pumpkin_util::{Difficulty, serde_enum_as_integer, world_seed::Seed};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod anvil;

// Constraint: disk biome palette serialization changed in 1.21.5
pub const MINIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4435; // 1.21.9
pub const MAXIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4556; // 1.21.10

pub const MINIMUM_SUPPORTED_LEVEL_VERSION: i32 = 19132; // 1.21.9
pub const MAXIMUM_SUPPORTED_LEVEL_VERSION: i32 = 19133; // 1.21.9

pub trait WorldInfoReader {
    fn read_world_info(&self, level_folder: &Path) -> Result<LevelData, WorldInfoError>;
}

pub trait WorldInfoWriter: Sync + Send {
    fn write_world_info(&self, info: &LevelData, level_folder: &Path)
    -> Result<(), WorldInfoError>;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct LevelData {
    // true if cheats are enabled.
    #[serde(rename = "allowCommands")]
    pub allow_commands: bool,
    // Center of the world border on the X coordinate. Defaults to 0.
    pub border_center_x: f64,
    // Center of the world border on the Z coordinate. Defaults to 0.
    pub border_center_z: f64,
    // Defaults to 0.2.
    pub border_damage_per_block: f64,
    // Width and length of the border of the border. Defaults to 60000000.
    pub border_size: f64,
    // Defaults to 5.
    pub border_safe_zone: f64,
    // Defaults to 60000000.
    pub border_size_lerp_target: f64,
    // Defaults to 0.
    pub border_size_lerp_time: i64,
    // Defaults to 5.
    pub border_warning_blocks: f64,
    // Defaults to 15.
    pub border_warning_time: f64,
    // The number of ticks until "clear weather" has ended.
    #[serde(rename = "clearWeatherTime")]
    pub clear_weather_time: i32,
    // TODO: Custom Boss Events

    // Options for data packs.
    pub data_packs: DataPacks,
    // An integer displaying the data version.
    pub data_version: i32,
    // The time of day. 0 is sunrise, 6000 is mid day, 12000 is sunset, 18000 is mid night, 24000 is the next day's 0. This value keeps counting past 24000 and does not reset to 0.
    pub day_time: i64,
    // The current difficulty setting.
    #[serde(with = "serde_enum_as_integer")]
    pub difficulty: Difficulty,
    // 1 or 0 (true/false) - True if the difficulty has been locked. Defaults to 0.
    pub difficulty_locked: bool,
    // TODO: DimensionData

    // Gamerules
    pub game_rules: GameRuleRegistry,

    // the generation settings for each dimension.
    pub world_gen_settings: WorldGenSettings,
    // The Unix time in milliseconds when the level was last loaded.
    pub last_played: i64,
    // The name of the level.
    pub level_name: String,
    // The X coordinate of the world spawn.
    pub spawn_x: i32,
    // The Y coordinate of the world spawn.
    pub spawn_y: i32,
    // The Z coordinate of the world spawn.
    pub spawn_z: i32,
    // The Yaw rotation of the world spawn.
    pub spawn_angle: f32,
    #[serde(rename = "Version")]
    pub world_version: WorldVersion,
    #[serde(rename = "version")]
    pub level_version: i32, // TODO: Implement the rest of the fields
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WorldGenSettings {
    // the numerical seed of the world
    pub seed: i64,
    pub dimensions: Dimensions,
}

pub type Dimensions = HashMap<String, Dimension>;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Dimension {
    pub generator: Generator,
    #[serde(rename = "type")]
    pub dimension_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Generator {
    pub settings: String,
    pub biome_source: BiomeSource,
    #[serde(rename = "type")]
    pub generator_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum BiomeSource {
    WithPreset {
        preset: String,
        #[serde(rename = "type")]
        biome_type: String,
    },
    Simple {
        #[serde(rename = "type")]
        biome_type: String,
    },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DataPacks {
    // List of disabled data packs.
    pub disabled: Vec<String>,
    // List of enabled data packs. By default, this is populated with a single string "vanilla".
    pub enabled: Vec<String>,
}

impl WorldGenSettings {
    #[must_use]
    pub fn new(seed: Seed) -> Self {
        // TODO: Adjust according to enabled worlds
        let mut dimensions = Dimensions::new();
        dimensions.insert(
            "minecraft:overworld".to_string(),
            Dimension {
                generator: Generator {
                    settings: "minecraft:overworld".to_string(),
                    biome_source: BiomeSource::WithPreset {
                        preset: "minecraft:overworld".to_string(),
                        biome_type: "minecraft:multi_noise".to_string(),
                    },
                    generator_type: "minecraft:noise".to_string(),
                },
                dimension_type: "minecraft:overworld".to_string(),
            },
        );
        dimensions.insert(
            "minecraft:the_nether".to_string(),
            Dimension {
                generator: Generator {
                    settings: "minecraft:nether".to_string(),
                    biome_source: BiomeSource::WithPreset {
                        preset: "minecraft:nether".to_string(),
                        biome_type: "minecraft:multi_noise".to_string(),
                    },
                    generator_type: "minecraft:noise".to_string(),
                },
                dimension_type: "minecraft:the_nether".to_string(),
            },
        );
        dimensions.insert(
            "minecraft:the_end".to_string(),
            Dimension {
                generator: Generator {
                    settings: "minecraft:end".to_string(),
                    biome_source: BiomeSource::Simple {
                        biome_type: "minecraft:the_end".to_string(),
                    },
                    generator_type: "minecraft:noise".to_string(),
                },
                dimension_type: "minecraft:the_end".to_string(),
            },
        );

        Self {
            dimensions,
            seed: seed.0 as i64,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct WorldVersion {
    // The version name as a string, e.g. "15w32b".
    pub name: String,
    // An integer displaying the data version.
    pub id: i32,
    // Whether the version is a snapshot or not.
    pub snapshot: bool,
    // Developing series. In 1.18 experimental snapshots, it was set to "ccpreview". In others, set to "main".
    pub series: String,
}

impl Default for WorldVersion {
    fn default() -> Self {
        Self {
            name: CURRENT_MC_VERSION.to_string(),
            id: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            snapshot: false,
            series: "main".to_string(),
        }
    }
}

impl LevelData {
    pub fn default(seed: Seed) -> Self {
        Self {
            allow_commands: true,
            border_center_x: 0.0,
            border_center_z: 0.0,
            border_damage_per_block: 0.2,
            border_size: 60_000_000.0,
            border_safe_zone: 5.0,
            border_size_lerp_target: 60_000_000.0,
            border_size_lerp_time: 0,
            border_warning_blocks: 5.0,
            border_warning_time: 15.0,
            clear_weather_time: -1,
            data_packs: DataPacks {
                disabled: vec![],
                enabled: vec!["vanilla".to_string()],
            },
            data_version: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            day_time: 0,
            difficulty: Difficulty::Normal,
            difficulty_locked: false,
            game_rules: GameRuleRegistry::default(),
            world_gen_settings: WorldGenSettings::new(seed),
            last_played: -1,
            level_name: "world".to_string(),
            spawn_x: 0,
            spawn_y: 200,
            spawn_z: 0,
            spawn_angle: 0.0,
            world_version: Default::default(),
            level_version: MAXIMUM_SUPPORTED_LEVEL_VERSION,
        }
    }
}

#[derive(Error, Debug)]
pub enum WorldInfoError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Info not found!")]
    InfoNotFound,
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Unsupported world data version: {0}")]
    UnsupportedDataVersion(i32),
    #[error("Unsupported world level version: {0}")]
    UnsupportedLevelVersion(i32),
}

impl From<std::io::Error> for WorldInfoError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::InfoNotFound,
            value => Self::IoError(value),
        }
    }
}
