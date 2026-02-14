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
pub const MAXIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4671; // 1.21.11

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
    #[serde(rename = "allowCommands", default)]
    pub allow_commands: bool,
    #[serde(default)]
    pub border_center_x: f64,
    #[serde(default)]
    pub border_center_z: f64,
    #[serde(default = "default_border_damage_per_block")]
    pub border_damage_per_block: f64,
    #[serde(default = "default_border_size")]
    pub border_size: f64,
    #[serde(default = "default_border_safe_zone")]
    pub border_safe_zone: f64,
    #[serde(default = "default_border_size")]
    pub border_size_lerp_target: f64,
    #[serde(default)]
    pub border_size_lerp_time: i64,
    #[serde(default = "default_border_warning_blocks")]
    pub border_warning_blocks: f64,
    #[serde(default = "default_border_warning_time")]
    pub border_warning_time: f64,
    #[serde(rename = "clearWeatherTime", default)]
    pub clear_weather_time: i32,
    #[serde(default = "default_data_packs")]
    pub data_packs: DataPacks,
    pub data_version: i32,
    #[serde(default)]
    pub day_time: i64,
    #[serde(with = "serde_enum_as_integer", default = "default_difficulty")]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub difficulty_locked: bool,
    #[serde(default)]
    pub game_rules: GameRuleRegistry,
    pub world_gen_settings: WorldGenSettings,
    #[serde(default)]
    pub last_played: i64,
    #[serde(default = "default_level_name")]
    pub level_name: String,
    #[serde(default)]
    pub spawn_x: i32,
    #[serde(default = "default_spawn_y")]
    pub spawn_y: i32,
    #[serde(default)]
    pub spawn_z: i32,
    #[serde(alias = "SpawnAngle", default)]
    pub spawn_yaw: f32,
    #[serde(default)]
    pub spawn_pitch: f32,
    #[serde(rename = "Version", default)]
    pub world_version: WorldVersion,
    #[serde(rename = "version", default = "default_level_version")]
    pub level_version: i32,
}

const DEFAULT_BORDER_DAMAGE_PER_BLOCK: f64 = 0.2;
const DEFAULT_BORDER_SIZE: f64 = 60_000_000.0;
const DEFAULT_BORDER_SAFE_ZONE: f64 = 5.0;
const DEFAULT_BORDER_WARNING_BLOCKS: f64 = 5.0;
const DEFAULT_BORDER_WARNING_TIME: f64 = 15.0;
const DEFAULT_DIFFICULTY: Difficulty = Difficulty::Normal;
const DEFAULT_LEVEL_NAME: &str = "world";
const DEFAULT_SPAWN_Y: i32 = 200;

fn default_border_damage_per_block() -> f64 { DEFAULT_BORDER_DAMAGE_PER_BLOCK }
fn default_border_size() -> f64 { DEFAULT_BORDER_SIZE }
fn default_border_safe_zone() -> f64 { DEFAULT_BORDER_SAFE_ZONE }
fn default_border_warning_blocks() -> f64 { DEFAULT_BORDER_WARNING_BLOCKS }
fn default_border_warning_time() -> f64 { DEFAULT_BORDER_WARNING_TIME }
fn default_data_packs() -> DataPacks {
    DataPacks { disabled: vec![], enabled: vec!["vanilla".to_string()] }
}
fn default_difficulty() -> Difficulty { DEFAULT_DIFFICULTY }
fn default_level_name() -> String { DEFAULT_LEVEL_NAME.to_string() }
fn default_spawn_y() -> i32 { DEFAULT_SPAWN_Y }
fn default_level_version() -> i32 { MAXIMUM_SUPPORTED_LEVEL_VERSION }

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
    pub settings: GeneratorSettings,
    #[serde(default)]
    pub biome_source: Option<BiomeSource>,
    #[serde(rename = "type")]
    pub generator_type: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum GeneratorSettings {
    Reference(String),
    Compound(pumpkin_nbt::compound::NbtCompound),
}

impl<'de> Deserialize<'de> for GeneratorSettings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct SettingsVisitor;

        impl<'de> serde::de::Visitor<'de> for SettingsVisitor {
            type Value = GeneratorSettings;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or compound")
            }

            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                Ok(GeneratorSettings::Reference(v.to_string()))
            }

            fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                Ok(GeneratorSettings::Reference(v))
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut compound = pumpkin_nbt::compound::NbtCompound::new();
                while let Some((key, value)) =
                    map.next_entry::<String, pumpkin_nbt::tag::NbtTag>()?
                {
                    compound.put(&key, value);
                }
                Ok(GeneratorSettings::Compound(compound))
            }
        }

        deserializer.deserialize_any(SettingsVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
                    settings: GeneratorSettings::Reference("minecraft:overworld".to_string()),
                    biome_source: Some(BiomeSource::WithPreset {
                        preset: "minecraft:overworld".to_string(),
                        biome_type: "minecraft:multi_noise".to_string(),
                    }),
                    generator_type: "minecraft:noise".to_string(),
                },
                dimension_type: "minecraft:overworld".to_string(),
            },
        );
        dimensions.insert(
            "minecraft:the_nether".to_string(),
            Dimension {
                generator: Generator {
                    settings: GeneratorSettings::Reference("minecraft:nether".to_string()),
                    biome_source: Some(BiomeSource::WithPreset {
                        preset: "minecraft:nether".to_string(),
                        biome_type: "minecraft:multi_noise".to_string(),
                    }),
                    generator_type: "minecraft:noise".to_string(),
                },
                dimension_type: "minecraft:the_nether".to_string(),
            },
        );
        dimensions.insert(
            "minecraft:the_end".to_string(),
            Dimension {
                generator: Generator {
                    settings: GeneratorSettings::Reference("minecraft:end".to_string()),
                    biome_source: Some(BiomeSource::Simple {
                        biome_type: "minecraft:the_end".to_string(),
                    }),
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
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
    #[must_use]
    pub fn default(seed: Seed) -> Self {
        Self {
            allow_commands: true,
            border_center_x: 0.0,
            border_center_z: 0.0,
            border_damage_per_block: DEFAULT_BORDER_DAMAGE_PER_BLOCK,
            border_size: DEFAULT_BORDER_SIZE,
            border_safe_zone: DEFAULT_BORDER_SAFE_ZONE,
            border_size_lerp_target: DEFAULT_BORDER_SIZE,
            border_size_lerp_time: 0,
            border_warning_blocks: DEFAULT_BORDER_WARNING_BLOCKS,
            border_warning_time: DEFAULT_BORDER_WARNING_TIME,
            clear_weather_time: -1,
            data_packs: default_data_packs(),
            data_version: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            day_time: 0,
            difficulty: DEFAULT_DIFFICULTY,
            difficulty_locked: false,
            game_rules: GameRuleRegistry::default(),
            world_gen_settings: WorldGenSettings::new(seed),
            last_played: -1,
            level_name: DEFAULT_LEVEL_NAME.to_string(),
            spawn_x: 0,
            spawn_y: DEFAULT_SPAWN_Y,
            spawn_z: 0,
            spawn_yaw: 0.0,
            spawn_pitch: 0.0,
            world_version: WorldVersion::default(),
            level_version: MAXIMUM_SUPPORTED_LEVEL_VERSION,
        }
    }

    pub const fn set_pos(&mut self, x: i32, z: i32) {
        self.spawn_x = x;
        self.spawn_z = z;
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
