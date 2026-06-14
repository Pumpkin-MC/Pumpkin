use serde::{Deserialize, Serialize};

/// Configuration for packet compression.
///
/// Controls whether network packet compression is enabled and the compression parameters.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct CompressionConfig {
    /// Whether compression is enabled.
    pub enabled: bool,
    /// Detailed compression settings.
    #[serde(flatten)]
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
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct CompressionInfo {
    /// The compression algorithm to use.
    /// Supported values: `"ZLib"`, `"Zstd"`. Defaults to `"ZLib"` when empty or unrecognized.
    pub compression: String,
    /// The maximum number of threads for Zstd compression.
    /// `0` means automatically determined based on available CPU cores.
    pub max_threads: u32,
    /// The compression threshold in bytes.
    /// Packets smaller than this will not be compressed.
    pub threshold: u32,
    /// Compression level, between `0..9`.
    /// `1` = optimize for speed, `9` = optimize for size.
    pub level: u32,
}

impl Default for CompressionInfo {
    fn default() -> Self {
        Self {
            compression: String::new(),
            max_threads: 0,
            threshold: 256,
            level: 4,
        }
    }
}
