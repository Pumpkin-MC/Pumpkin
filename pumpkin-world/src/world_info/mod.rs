use pumpkin_config::BASIC_CONFIG;
use pumpkin_util::Difficulty;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{generation::Seed, level::LevelFolder};

pub mod anvil;

pub const MINIMUM_SUPPORTED_WORLD_DATA_VERSION: u32 = 4080; // 1.21.2
pub const MAXIMUM_SUPPORTED_WORLD_DATA_VERSION: u32 = 4189; // 1.21.4

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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LevelData {
    // An integer displaying the data version.
    pub data_version: u32,
    // The current difficulty setting.
    pub difficulty: u8,
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

#[derive(Serialize, Deserialize, Clone)]
pub struct WorldGenSettings {
    // the numerical seed of the world
    pub seed: i64,
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct WorldVersion {
    // The version name as a string, e.g. "15w32b".
    pub name: String,
    // An integer displaying the data version.
    pub id: i32,
    // Whether the version is a snapshot or not.
    // TODO: fast_nbt doesnt support bools
    // pub snapshot: bool,

    // Developing series. In 1.18 experimental snapshots, it was set to "ccpreview". In others, set to "main".
    pub series: String,
}

impl Default for WorldVersion {
    fn default() -> Self {
        Self {
            name: "1.24.4".to_string(),
            id: -1,
            series: "main".to_string(),
        }
    }
}

impl Default for LevelData {
    fn default() -> Self {
        Self {
            // TODO: Define defaults somewhere
            data_version: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            difficulty: Difficulty::Normal as u8,
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
    UnsupportedVersion(u32),
}

impl From<std::io::Error> for WorldInfoError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::InfoNotFound,
            value => Self::IoError(value),
        }
    }
}
