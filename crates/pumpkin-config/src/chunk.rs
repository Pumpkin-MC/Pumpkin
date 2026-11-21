use std::str;

use serde::{Deserialize, Serialize};

/// Configuration for chunk storage format.
///
/// Supports `Anvil` (`.mca`), `Linear` (`.linear`), and `Pump` (`.pump`).
#[derive(Deserialize, Serialize, Clone, Default)]
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

// Spanner's internally tagged derive does not support newtype variants.
impl<'de> toml_spanner::FromToml<'de> for ChunkConfig {
    fn from_toml(
        ctx: &mut toml_spanner::Context<'de>,
        item: &toml_spanner::Item<'de>,
    ) -> Result<Self, toml_spanner::Failed> {
        let table = item.require_table(ctx)?;
        let tag = table
            .get("type")
            .ok_or_else(|| ctx.report_missing_field("type", item))?;
        match tag.require_string(ctx)? {
            "anvil" => AnvilChunkConfig::from_toml(ctx, item).map(Self::Anvil),
            "linear" => Ok(Self::Linear),
            "pump" => Ok(Self::Pump),
            _ => Err(ctx.report_unexpected_variant(&["anvil", "linear", "pump"], tag)),
        }
    }
}

/// Configuration for Anvil chunk storage.
#[derive(Deserialize, Serialize, Default, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct AnvilChunkConfig {
    /// Compression settings for chunk data.
    #[toml(default = Self::default().compression)]
    pub compression: ChunkCompression,
}

/// Compression settings for chunk data.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct ChunkCompression {
    /// Compression algorithm to use.
    #[toml(default = Self::default().algorithm)]
    pub algorithm: Compression,
    /// Compression level (algorithm-specific).
    #[toml(default = Self::default().level)]
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

/// Compression algorithm used for chunk data storage.
#[derive(Deserialize, Serialize, Clone, Copy, toml_spanner::Toml)]
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
