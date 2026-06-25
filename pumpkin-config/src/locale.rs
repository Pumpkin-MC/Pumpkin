use serde::{Deserialize, Serialize};

/// Configuration for locale and language resolution behaviour.
///
/// Controls how the server determines which language to use for
/// player-facing messages, command output, and server logs.
///
/// # Options
/// * `"auto"` — auto-detect from the player's client settings or system environment.
/// * `"zh_cn"`, `"ja_jp"`, etc. — force a specific locale, skipping detection.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
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
    /// Language used for command output in the server console.
    ///
    /// `"auto"` detects the system locale from environment variables.
    /// A specific code forces command output to use that language.
    pub server_command: String,
    /// Language used for server log messages.
    ///
    /// `"auto"` detects the system locale from environment variables.
    /// A specific code forces log output to use that language.
    pub server_logging: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            client_java_edition: "auto".to_string(),
            client_bedrock_edition: "auto".to_string(),
            server_command: "auto".to_string(),
            server_logging: "auto".to_string(),
        }
    }
}
