use std::str;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ChunkConfig {
    pub compression: ChunkCompression,
}

#[derive(Deserialize, Serialize)]
pub struct ChunkCompression {
    pub compression_algorithm: Compression,
    pub compression_level: u32,
}

impl Default for ChunkCompression {
    fn default() -> Self {
        Self {
            compression_algorithm: Compression::LZ4,
            compression_level: 6,
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[repr(u8)]
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
