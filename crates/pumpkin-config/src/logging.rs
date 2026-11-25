use serde::{Deserialize, Serialize};

/// Configuration for server logging behavior.
///
/// Controls log output, formatting, and file settings.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct LoggingConfig {
    /// Whether logging is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Minimum log level for console and file output ("trace", "debug", "info", "warn", "error", "off").
    #[toml(default = Self::default().level)]
    pub level: String,
    /// Whether to include thread names in log output.
    #[toml(default = Self::default().threads)]
    pub threads: bool,
    /// Whether to include thread IDs in log output.
    #[toml(default = Self::default().thread_ids)]
    pub thread_ids: bool,
    /// Whether to include target (module/component path) in log output.
    #[toml(default = Self::default().target)]
    pub target: bool,
    /// Whether to enable coloured log output.
    #[toml(default = Self::default().color)]
    pub color: bool,
    /// Whether to include timestamps in log entries.
    #[toml(default = Self::default().timestamp)]
    pub timestamp: bool,
    /// Format description for timestamps (using `time` format description syntax).
    #[toml(default = Self::default().timestamp_format)]
    pub timestamp_format: String,
    /// Path to the log file.
    #[toml(default = Self::default().file)]
    pub file: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            threads: false,
            thread_ids: false,
            target: false,
            color: true,
            timestamp: true,
            timestamp_format: "[hour]:[minute]:[second]".to_string(),
            file: "latest.log".to_string(),
        }
    }
}
