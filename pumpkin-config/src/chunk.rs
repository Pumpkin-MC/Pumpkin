use std::str;

use serde::{Deserialize, Serialize};

/// Configuration for chunk storage format.
///
/// Supports multiple chunk formats, currently `Anvil` and `Linear`.
#[derive(Deserialize, Default, Serialize, Clone)]
#[serde(tag = "type")]
pub enum ChunkConfig {
    /// Standard Anvil chunk storage.
    #[serde(rename = "anvil")]
    Anvil(AnvilChunkConfig),
    /// Linear chunk storage format.
    #[serde(rename = "linear")]
    Linear,
    /// Pumpkin's own optimized world format.
    #[serde(rename = "pump")]
    #[default]
    Pump,
}

/// Configuration for Anvil chunk storage.
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct AnvilChunkConfig {
    /// Compression settings for chunk data.
    pub compression: ChunkCompression,
    /// Whether chunks should be written in place.
    pub write_in_place: bool,
}

/// Compression settings for chunk data.
#[derive(Deserialize, Serialize, Clone)]
pub struct ChunkCompression {
    /// Compression algorithm to use.
    pub algorithm: Compression,
    /// Compression level (algorithm-specific).
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
    /// `GZip` Compression.
    GZip,
    /// `ZLib` Compression.
    ZLib,
    /// LZ4 Compression (since 24w04a).
    LZ4,
    /// Custom compression algorithm (since 24w05a).
    Custom,
}
