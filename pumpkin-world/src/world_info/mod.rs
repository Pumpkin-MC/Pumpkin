use pumpkin_config::BASIC_CONFIG;
use pumpkin_util::Difficulty;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{generation::Seed, level::LevelFolder};

pub mod anvil;

pub const MINIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4080; // 1.21.2
pub const MAXIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4189; // 1.21.4

pub(crate) trait WorldInfoReader {
    fn read_world_info(&self, level_folder: &LevelFolder) -> Result<LevelData, WorldInfoError>;
}

pub(crate) trait WorldInfoWriter: Sync + Send {
    fn write_world_info(
        &self,
        info: LevelData,
        level_folder: &LevelFolder,
    ) -> Result<(), WorldInfoError>;
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
    pub difficulty: i8,
    // 1 or 0 (true/false) - True if the difficulty has been locked. Defaults to 0.
    pub difficulty_locked: bool,
    // TODO: DimensionData

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
    #[serde(rename = "version")]
    // The NBT version of the level
    pub nbt_version: i32,
    #[serde(rename = "Version")]
    pub version: WorldVersion,
    // TODO: Implement the rest of the fields
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WorldGenSettings {
    // the numerical seed of the world
    pub seed: i64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DataPacks {
    // List of disabled data packs.
    pub disabled: Vec<String>,
    // List of enabled data packs. By default, this is populated with a single string "vanilla".
    pub enabled: Vec<String>,
}

fn get_or_create_seed() -> Seed {
    // TODO: if there is a seed in the config (!= "") use it. Otherwise make a random one
    Seed::from(BASIC_CONFIG.seed.as_str())
}

impl Default for WorldGenSettings {
    fn default() -> Self {
        Self {
            seed: get_or_create_seed().0 as i64,
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
            name: "1.24.4".to_string(),
            id: -1,
            snapshot: false,
            series: "main".to_string(),
        }
    }
}

impl Default for LevelData {
    fn default() -> Self {
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
            difficulty: Difficulty::Normal as i8,
            difficulty_locked: false,
            world_gen_settings: Default::default(),
            last_played: -1,
            level_name: "world".to_string(),
            spawn_x: 0,
            spawn_y: 200,
            spawn_z: 0,
            spawn_angle: 0.0,
            nbt_version: -1,
            version: Default::default(),
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
    UnsupportedVersion(i32),
}

impl From<std::io::Error> for WorldInfoError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::InfoNotFound,
            value => Self::IoError(value),
        }
    }
}
