use std::str;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct ChunkConfig {
    pub compression: ChunkCompression,
    pub format: ChunkFormat,
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

#[derive(Deserialize, Serialize, Clone)]
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

#[derive(Deserialize, Serialize, Clone, Default)]
pub enum ChunkFormat {
    #[default]
    Anvil,
    Linear,
}
