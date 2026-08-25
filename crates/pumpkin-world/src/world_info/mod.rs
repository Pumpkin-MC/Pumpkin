use std::collections::HashMap;
use std::path::Path;

use crate::CURRENT_MC_VERSION;
use pumpkin_data::game_rules::GameRuleRegistry;
use pumpkin_util::{Difficulty, serde_enum_as_integer, world_seed::Seed};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod anvil;
pub mod data_files;

// Constraint: disk biome palette serialization changed in 1.21.5
pub const MINIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4435; // 1.21.9
pub const MAXIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4903; // 26.2

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
    #[serde(default = "default_data_packs")]
    pub data_packs: DataPacks,
    pub data_version: i32,
    #[serde(with = "serde_enum_as_integer", default = "default_difficulty")]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub difficulty_locked: bool,
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
    #[serde(rename = "map_id", default)]
    pub map_id: i32,

    // These are NOT serialized to level.dat, but are still deserialized from it if present.
    // They are loaded and saved by AnvilLevelInfo via the data_files module.
    /// Game rules – persisted to `data/minecraft/game_rules.dat`.
    #[serde(skip_serializing, default)]
    pub game_rules: GameRuleRegistry,

    /// World generation settings – persisted to `data/minecraft/world_gen_settings.dat`.
    #[serde(skip_serializing, default)]
    pub world_gen_settings: WorldGenSettings,

    /// In-game time of day (overworld dimension clock).
    /// Persisted to `data/minecraft/world_clocks.dat`.
    #[serde(skip_serializing, default)]
    pub day_time: i64,

    /// Remaining ticks of forced-clear weather.
    /// Persisted to `data/minecraft/weather.dat`.
    #[serde(rename = "clearWeatherTime", skip_serializing, default)]
    pub clear_weather_time: i32,
}

const DEFAULT_BORDER_DAMAGE_PER_BLOCK: f64 = 0.2;
const DEFAULT_BORDER_SIZE: f64 = 60_000_000.0;
const DEFAULT_BORDER_SAFE_ZONE: f64 = 5.0;
const DEFAULT_BORDER_WARNING_BLOCKS: f64 = 5.0;
const DEFAULT_BORDER_WARNING_TIME: f64 = 15.0;
const DEFAULT_DIFFICULTY: Difficulty = Difficulty::Normal;
const DEFAULT_LEVEL_NAME: &str = "world";
const DEFAULT_SPAWN_Y: i32 = 200;
const DEFAULT_ENABLED_DATA_PACK: &str = "vanilla";
const DEFAULT_WORLD_VERSION_SERIES: &str = "main";

const fn default_border_damage_per_block() -> f64 {
    DEFAULT_BORDER_DAMAGE_PER_BLOCK
}
const fn default_border_size() -> f64 {
    DEFAULT_BORDER_SIZE
}
const fn default_border_safe_zone() -> f64 {
    DEFAULT_BORDER_SAFE_ZONE
}
const fn default_border_warning_blocks() -> f64 {
    DEFAULT_BORDER_WARNING_BLOCKS
}
const fn default_border_warning_time() -> f64 {
    DEFAULT_BORDER_WARNING_TIME
}
fn default_enabled_data_packs() -> Vec<String> {
    vec![DEFAULT_ENABLED_DATA_PACK.to_string()]
}
fn default_data_packs() -> DataPacks {
    DataPacks {
        disabled: vec![],
        enabled: default_enabled_data_packs(),
    }
}
const fn default_difficulty() -> Difficulty {
    DEFAULT_DIFFICULTY
}
fn default_level_name() -> String {
    DEFAULT_LEVEL_NAME.to_string()
}
const fn default_spawn_y() -> i32 {
    DEFAULT_SPAWN_Y
}
const fn default_level_version() -> i32 {
    MAXIMUM_SUPPORTED_LEVEL_VERSION
}
fn default_world_version_name() -> String {
    CURRENT_MC_VERSION.to_string()
}
const fn default_world_version_id() -> i32 {
    MAXIMUM_SUPPORTED_WORLD_DATA_VERSION
}
fn default_world_version_series() -> String {
    DEFAULT_WORLD_VERSION_SERIES.to_string()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct WorldGenSettings {
    // the numerical seed of the world
    pub seed: i64,
    #[serde(default)]
    pub dimensions: Dimensions,
}

impl Default for WorldGenSettings {
    fn default() -> Self {
        // Use seed 0 as placeholder; actual seed comes from config or world_gen_settings.dat
        Self::new(pumpkin_util::world_seed::Seed(0))
    }
}

pub type Dimensions = HashMap<String, Dimension>;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Dimension {
    pub generator: Generator,
    #[serde(rename = "type")]
    pub dimension_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Generator {
    #[serde(default)]
    pub settings: Option<GeneratorSettings>,
    #[serde(default)]
    pub biome_source: Option<BiomeSource>,
    #[serde(rename = "type")]
    pub generator_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum GeneratorSettings {
    Reference(String),
    Compound(serde_json::Value),
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
    #[serde(default)]
    pub disabled: Vec<String>,
    // List of enabled data packs. By default, this is populated with a single string "vanilla".
    #[serde(default = "default_enabled_data_packs")]
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
                    settings: Some(GeneratorSettings::Reference(
                        "minecraft:overworld".to_string(),
                    )),
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
                    settings: Some(GeneratorSettings::Reference("minecraft:nether".to_string())),
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
                    settings: Some(GeneratorSettings::Reference("minecraft:end".to_string())),
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
    #[serde(default = "default_world_version_name")]
    pub name: String,
    // An integer displaying the data version.
    #[serde(default = "default_world_version_id")]
    pub id: i32,
    // Whether the version is a snapshot or not.
    #[serde(default)]
    pub snapshot: bool,
    // Developing series. In 1.18 experimental snapshots, it was set to "ccpreview". In others, set to "main".
    #[serde(default = "default_world_version_series")]
    pub series: String,
}

impl Default for WorldVersion {
    fn default() -> Self {
        Self {
            name: default_world_version_name(),
            id: default_world_version_id(),
            snapshot: false,
            series: default_world_version_series(),
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
            data_packs: default_data_packs(),
            data_version: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            difficulty: DEFAULT_DIFFICULTY,
            difficulty_locked: false,
            last_played: -1,
            level_name: DEFAULT_LEVEL_NAME.to_string(),
            spawn_x: 0,
            spawn_y: DEFAULT_SPAWN_Y,
            spawn_z: 0,
            spawn_yaw: 0.0,
            spawn_pitch: 0.0,
            world_version: WorldVersion::default(),
            level_version: MAXIMUM_SUPPORTED_LEVEL_VERSION,
            map_id: 0,
            // fields now in data/minecraft/*.dat
            game_rules: GameRuleRegistry::default(),
            world_gen_settings: WorldGenSettings::new(seed),
            day_time: 0,
            clear_weather_time: -1,
        }
    }

    #[must_use]
    pub fn from_world_generator(
        seed: Seed,
        generator: &crate::generation::generator::VanillaGenerator,
    ) -> Self {
        let mut data = Self::default(seed);
        let spawn_pos = generator.find_spawn_position();
        data.spawn_x = spawn_pos.0.x;
        data.spawn_z = spawn_pos.0.z;
        data
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
    #[error(
        "No world seed found: neither level.dat nor data/minecraft/world_gen_settings.dat contains one"
    )]
    MissingWorldSeed,
    #[error("Serialization error: {0}")]
    SerializationError(String),
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
