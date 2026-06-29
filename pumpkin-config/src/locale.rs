use serde::{Deserialize, Serialize};

/// Configuration for locale and language resolution behaviour.
///
/// Controls how the server determines which language to use for
/// player-facing messages, command output, and server logs.
///
/// # Options
/// * `"auto"` — auto-detect from the player's client settings or system environment.
/// * `"zh-CN"`, `"ja-JP"`, etc. — force a specific locale, skipping detection.
///
/// All fields have sensible defaults (see [`Default`] impl).
/// Struct-level `#[serde(default)]` ensures missing fields in the TOML
/// config fall back to [`Default::default`].
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
    /// Base URL of the translation mirror. When empty, the default
    /// resource server is used.
    ///
    /// Set this to a closer mirror to speed up translation downloads.
    pub mirror_url: String,
    /// Download timeout per translation file request, in milliseconds.
    ///
    /// When any download exceeds this timeout, the server falls back
    /// to compile-time embedded English translations.
    /// Default: `10000` (10 s).
    pub timeout: u64,
    /// Skip SHA256 checksum verification for downloaded translation files.
    ///
    /// When `true`, downloaded files are accepted without hash verification.
    /// Useful when the mirror does not host `.sha256` files.
    /// Default: `false`.
    pub skip_checksum: bool,
    /// Directory where downloaded translation files are cached on disk.
    ///
    /// If empty or relative, it is resolved against the server's working
    /// directory. An absolute path is used as-is.
    /// Default: `"data/translation"`.
    pub translation_cache_dir: String,
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
            mirror_url: String::new(),
            timeout: 10000,
            skip_checksum: false,
            translation_cache_dir: "data/translation".to_string(),
            server_global: "auto".to_string(),
            client_java_edition: "auto".to_string(),
            client_bedrock_edition: "auto".to_string(),
        }
    }
}
