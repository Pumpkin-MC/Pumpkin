use serde::{Deserialize, Serialize};

/// Controls which languages the server uses for commands and log output.
///
/// Set to `"auto"` (the default) to let Pumpkin detect the system language
/// at startup. Set to a specific locale string (e.g. `"zh_cn"`, `"de_de"`)
/// to force a particular language. Unrecognised strings fall back to `en_us`.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct LocaleConfig {
    /// Language used when displaying command output to players and console.
    /// Default: `"auto"`.
    pub command: String,
    /// Language used for log records written by the server.
    /// Default: `"auto"`.
    pub log: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            command: "auto".to_string(),
            log: "auto".to_string(),
        }
    }
}
