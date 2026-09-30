use serde::{Deserialize, Serialize};

/// Configuration for server logging behavior.
///
/// Controls log output, formatting, and file settings.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Whether logging is enabled.
    pub enabled: bool,
    /// Minimum log level for console and file output ("trace", "debug", "info", "warn", "error", "off").
    pub level: String,
    /// Whether to include thread names in log output.
    pub threads: bool,
    /// Whether to include thread IDs in log output.
    pub thread_ids: bool,
    /// Whether to include target (module/component path) in log output.
    pub target: bool,
    /// Whether to enable coloured log output.
    pub color: bool,
    /// Whether to include timestamps in log entries.
    pub timestamp: bool,
    /// Format description for timestamps (using `time` format description syntax).
    pub timestamp_format: String,
    /// Path to the log file.
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
