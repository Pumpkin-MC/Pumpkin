pub mod anvil_saver;

use pumpkin_util::math::vector2::Vector2;
use thiserror::Error;

use crate::level::LevelFolder;

use super::{compression::CompressionError, ChunkParsingError};

pub trait ChunkStorage: Sync + Send {
    fn read_raw_chunk(
        &self,
        save_file: &LevelFolder,
        at: &Vector2<i32>,
    ) -> Result<Vec<u8>, ChunkStorageReadingError>;

    fn write_raw_chunk(
        &self,
        chunk: Vec<u8>,
        level_folder: &LevelFolder,
        at: &Vector2<i32>,
    ) -> Result<(), ChunkStorageWritingError>;
}

#[derive(Error, Debug)]
pub enum ChunkStorageReadingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Invalid header")]
    InvalidHeader,
    #[error("Region is invalid")]
    RegionIsInvalid,
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Tried to read chunk which does not exist")]
    ChunkNotExist,
    #[error("Failed to parse Chunk from bytes: {0}")]
    ParsingError(ChunkParsingError),
}

#[derive(Error, Debug)]
pub enum ChunkStorageWritingError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Compression error {0}")]
    Compression(CompressionError),
    #[error("Chunk serializing error: {0}")]
    ChunkSerializingError(String),
}
