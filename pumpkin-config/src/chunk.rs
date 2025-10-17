use std::str;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ChunkConfig {
    #[serde(rename = "anvil")]
    Anvil(AnvilChunkConfig),
    #[serde(rename = "linear")]
    Linear(LinearChunkConfig),
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self::Anvil(Default::default())
    }
}

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct AnvilChunkConfig {
    pub compression: ChunkCompression,
    pub write_in_place: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ChunkCompression {
    pub algorithm: Compression,
    pub level: u32,
}

impl Default for ChunkCompression {
    fn default() -> Self {
        Self {
            algorithm: Compression::LZ4,
            level: 6,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum Compression {
    /// GZip Compression
    GZip,
    /// ZLib Compression
    ZLib,
    /// LZ4 Compression (since 24w04a)
    LZ4,
    /// Custom compression algorithm (since 24w05a)
    Custom,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub struct LinearChunkConfig {
    pub linear_version: LinearVersion,
}

#[derive(Deserialize, Serialize, Default, Clone)]
pub enum LinearVersion {
    #[default]
    V1,
    // TODO: V2,
}
