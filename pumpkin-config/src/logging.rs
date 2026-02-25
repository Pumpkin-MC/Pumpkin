use serde::{Deserialize, Serialize};

/// Configuration for server logging behavior.
///
/// Controls log output, formatting, and file settings.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Whether logging is enabled.
    pub enabled: bool,
    /// Whether to include thread names and IDs in log output.
    pub threads: bool,
    /// Whether to include module paths in log output.
    pub target: bool,
    /// Whether to enable coloured log output.
    pub color: bool,
    /// Whether to include timestamps in log entries.
    pub timestamp: bool,
    /// Path to the log file.
    pub file: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threads: false,
            target: false,
            color: true,
            timestamp: true,
            file: "latest.log".to_string(),
        }
    }
}
