use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Runtime flag: verbose diagnostic logs for development.
/// Set from [`LoggingConfig::development`] at server start.
static DEVELOPMENT_MODE: AtomicBool = AtomicBool::new(false);

/// Enable or disable development diagnostic logging at runtime.
pub fn set_development_mode(enabled: bool) {
    DEVELOPMENT_MODE.store(enabled, Ordering::Relaxed);
}

/// Whether development diagnostic logging is enabled.
///
/// When `false` (default), only the previous quieter log paths run (mostly
/// `debug!` / `trace!`). When `true`, extra INFO/WARN diagnostics for tick lag,
/// terrain generation, chunk loads, and duplicate logins are emitted.
#[must_use]
pub fn development_mode() -> bool {
    DEVELOPMENT_MODE.load(Ordering::Relaxed)
}

/// Configuration for server logging behavior.
///
/// Controls log output, formatting, and file settings.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LoggingConfig {
    /// Whether logging is enabled.
    pub enabled: bool,
    /// Whether to include thread names in log output.
    pub threads: bool,
    /// Whether to enable coloured log output.
    pub color: bool,
    /// Whether to include timestamps in log entries.
    pub timestamp: bool,
    /// Path to the log file.
    pub file: String,
    /// Extra diagnostic logs for development (slow ticks, terrain gen, etc.).
    ///
    /// Default `false` keeps production-style quiet logs. Set `true` while
    /// debugging freezes, generation, or reconnect issues.
    pub development: bool,
    /// Server console language / 服务端控制台语言.
    ///
    /// Supported values:
    /// - `en_us` — English (default)
    /// - `zh_cn` — 简体中文
    /// - `zh_en` / `bilingual` — 中英双语 (Chinese / English side-by-side in console)
    ///
    /// Player chat/UI still follows each client's own language setting.
    /// 玩家客户端界面仍跟随各自客户端语言设置。
    pub locale: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threads: true,
            color: true,
            timestamp: true,
            file: "latest.log".to_string(),
            development: false,
            locale: "en_us".to_string(),
        }
    }
}
