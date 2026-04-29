use std::path::Path;

use thiserror::Error;

pub use pumpkin_storage::world_info::{
    BiomeSource, DataPacks, Dimension, Dimensions, Generator, GeneratorSettings, LevelData,
    MAXIMUM_SUPPORTED_LEVEL_VERSION, MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
    MINIMUM_SUPPORTED_LEVEL_VERSION, MINIMUM_SUPPORTED_WORLD_DATA_VERSION, WorldGenSettings,
    WorldVersion,
};

pub mod anvil;

pub trait WorldInfoReader {
    fn read_world_info(&self, level_folder: &Path) -> Result<LevelData, WorldInfoError>;
}

pub trait WorldInfoWriter: Sync + Send {
    fn write_world_info(&self, info: &LevelData, level_folder: &Path)
    -> Result<(), WorldInfoError>;
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
