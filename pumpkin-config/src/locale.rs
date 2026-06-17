use serde::{Deserialize, Serialize};

/// Configuration for server-side translation / locale behaviour.
///
/// Controls which language is used for **client translations** (per-edition),
/// **command feedback**, and **log output**.  Each field accepts either
/// `"auto"` (auto-detect) or a specific locale identifier such as `"zh_cn"`.
///
/// Processing logic lives in [`pumpkin_i18n::client`] and
/// [`pumpkin_i18n::server`].
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
    /// Locale for Java Edition client translations.
    pub client_java_edition: String,
    /// Locale for Bedrock Edition client translations.
    pub client_bedrock_edition: String,
    /// Locale for command feedback (console, RCON, command blocks).
    pub server_command: String,
    /// Locale for server log messages.
    pub server_logging: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            client_java_edition: String::from("auto"),
            client_bedrock_edition: String::from("auto"),
            server_command: String::from("auto"),
            server_logging: String::from("auto"),
        }
    }
}

impl LocaleConfig {
    /// Validates every field, logging a warning for unrecognised values.
    pub fn validate(&self) {
        pumpkin_i18n::server::validate_locale_config(&self.server_command, &self.server_logging);
        pumpkin_i18n::client::validate_locale_config(
            &self.client_java_edition,
            &self.client_bedrock_edition,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto() {
        let cfg = LocaleConfig::default();
        assert_eq!(cfg.client_java_edition, "auto");
        assert_eq!(cfg.client_bedrock_edition, "auto");
        assert_eq!(cfg.server_command, "auto");
        assert_eq!(cfg.server_logging, "auto");
    }

    #[test]
    fn toml_roundtrip_default() {
        let cfg = LocaleConfig::default();
        let s = toml::to_string(&cfg).expect("serialize");
        let restored: LocaleConfig = toml::from_str(&s).expect("deserialize");
        assert_eq!(restored.client_java_edition, "auto");
        assert_eq!(restored.client_bedrock_edition, "auto");
    }

    #[test]
    fn toml_missing_field_uses_default() {
        let cfg: LocaleConfig =
            toml::from_str("client_java_edition = \"fr_fr\"\n").expect("deserialize");
        assert_eq!(cfg.client_java_edition, "fr_fr");
        assert_eq!(cfg.client_bedrock_edition, "auto");
    }

    #[test]
    fn toml_empty_uses_defaults() {
        let cfg: LocaleConfig = toml::from_str("").expect("deserialize");
        assert_eq!(cfg.client_java_edition, "auto");
    }
}
