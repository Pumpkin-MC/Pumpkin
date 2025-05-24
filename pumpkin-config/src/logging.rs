use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy)]
pub enum LoggingLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub enabled: bool,
    pub level: LoggingLevel,
    pub env: bool,
    pub threads: bool,
    pub color: bool,
    pub timestamp: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: LoggingLevel::Info,
            env: false,
            threads: true,
            color: true,
            timestamp: true,
        }
    }
}

impl From<LoggingLevel> for LevelFilter {
    fn from(value: LoggingLevel) -> Self {
        match value {
            LoggingLevel::Off => LevelFilter::Off,
            LoggingLevel::Error => LevelFilter::Error,
            LoggingLevel::Warn => LevelFilter::Warn,
            LoggingLevel::Info => LevelFilter::Info,
            LoggingLevel::Debug => LevelFilter::Debug,
            LoggingLevel::Trace => LevelFilter::Trace,
        }
    }
}
