use serde::{Deserialize, Serialize};

/// Configuration for locale and language resolution behaviour.
///
/// Controls how the server determines which language to use for
/// player-facing messages, command output, and server logs.
///
/// # Options
/// * `"auto"` — auto-detect from the player's client settings or system environment.
/// * `"zh-CN"`, `"ja-JP"`, etc. — force a specific locale, skipping detection.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
    /// Language used for server logs and console/backend command output.
    ///
    /// `"auto"` detects the system locale from environment variables or the
    /// platform locale API. A specific code forces server-side output to use
    /// that language. This does not affect client display.
    pub server_global: String,
    /// Language resolution for Java Edition players.
    ///
    /// `"auto"` reads the locale reported by the Java client.
    /// A specific code forces that language for all Java players.
    pub client_java_edition: String,
    /// Language resolution for Bedrock Edition players.
    ///
    /// `"auto"` reads the locale reported by the Bedrock client.
    /// A specific code forces that language for all Bedrock players.
    pub client_bedrock_edition: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            server_global: "auto".to_string(),
            client_java_edition: "auto".to_string(),
            client_bedrock_edition: "auto".to_string(),
        }
    }
}
