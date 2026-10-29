use serde::{Deserialize, Serialize};

/// Configuration for packet compression.
///
/// Controls whether network packet compression is enabled and the compression parameters.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct CompressionConfig {
    /// Whether compression is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Detailed compression settings.
    #[serde(flatten)]
    #[toml(flatten, with = toml_spanner::helper::flatten_any)]
    pub info: CompressionInfo,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            info: CompressionInfo::default(),
        }
    }
}

/// Detailed information for packet compression settings.
///
/// Can also be used independently of the config.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct CompressionInfo {
    /// The compression threshold in bytes.
    /// Packets smaller than this will not be compressed.
    #[toml(default = Self::default().threshold)]
    pub threshold: u32,
    /// Compression level, between `0..9`.
    /// `1` = optimize for speed, `9` = optimize for size.
    #[toml(default = Self::default().level)]
    pub level: u32,
}

impl Default for CompressionInfo {
    fn default() -> Self {
        Self {
            threshold: 256,
            level: 4,
        }
    }
}
