use std::str::FromStr;

use pumpkin_i18n::Locale;
use serde::{Deserialize, Serialize};

/// Configuration for server-side translation / locale behaviour.
///
/// Controls which language is used for **client translations** (per-edition),
/// **command feedback**, and **log output**.  Each field accepts either
/// `"auto"` (auto-detect) or a specific locale identifier such as `"zh_cn"`.
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
    // ------------------------------------------------------------------
    // Client helpers — delegate to pumpkin_i18n::client
    // ------------------------------------------------------------------

    /// Resolves and caches the locale for a Java Edition player.
    #[must_use]
    pub fn set_java_client_locale(&self, player_id: &str, player_lang: &str) -> Locale {
        pumpkin_i18n::client::set_player_locale(player_id, player_lang, &self.client_java_edition)
    }

    /// Resolves and caches the locale for a Bedrock Edition player.
    #[must_use]
    pub fn set_bedrock_client_locale(&self, player_id: &str, player_lang: &str) -> Locale {
        pumpkin_i18n::client::set_player_locale(
            player_id,
            player_lang,
            &self.client_bedrock_edition,
        )
    }

    /// Returns the cached locale for a player.
    #[must_use]
    pub fn get_client_locale(&self, player_id: &str) -> Locale {
        pumpkin_i18n::client::get_player_locale(player_id).unwrap_or(Locale::EnUs)
    }

    /// Removes a player's cached locale (call on disconnect).
    pub fn remove_client_locale(&self, player_id: &str) {
        pumpkin_i18n::client::remove_player_locale(player_id);
    }

    // ------------------------------------------------------------------
    // Server helpers — delegate to pumpkin_i18n::server
    // ------------------------------------------------------------------

    /// Resolves the locale for command feedback.
    #[must_use]
    pub fn command_locale(&self) -> Locale {
        pumpkin_i18n::server::resolve_server_locale(&self.server_command)
    }

    /// Resolves the locale for log output.
    #[must_use]
    pub fn logging_locale(&self) -> Locale {
        pumpkin_i18n::server::resolve_server_locale(&self.server_logging)
    }

    // ------------------------------------------------------------------
    // Validation
    // ------------------------------------------------------------------

    /// Validates every field, logging a warning for unrecognised values.
    pub fn validate(&self) {
        let fields: [(&str, &str); 4] = [
            ("client_java_edition", &self.client_java_edition),
            ("client_bedrock_edition", &self.client_bedrock_edition),
            ("server_command", &self.server_command),
            ("server_logging", &self.server_logging),
        ];
        for (label, value) in fields {
            if value.eq_ignore_ascii_case("auto") || value.is_empty() {
                continue;
            }
            if pumpkin_i18n::Locale::from_str(value).is_err() {
                tracing::warn!(
                    "[locale].{label} = \"{value}\" is not a recognised locale – \
                     falling back to English (en_us)"
                );
            }
        }
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
    fn java_client_auto_uses_player_lang() {
        let cfg = LocaleConfig::default();
        assert_eq!(cfg.set_java_client_locale("u1", "zh_cn"), Locale::ZhCn);
    }

    #[test]
    fn forced_edition_overrides_player() {
        let cfg = LocaleConfig {
            client_java_edition: "fr_fr".into(),
            ..Default::default()
        };
        assert_eq!(cfg.set_java_client_locale("u2", "zh_cn"), Locale::FrFr);
    }

    #[test]
    fn server_explicit_locale() {
        let cfg = LocaleConfig {
            server_command: "ja_jp".into(),
            server_logging: "ko_kr".into(),
            ..Default::default()
        };
        assert_eq!(cfg.command_locale(), Locale::JaJp);
        assert_eq!(cfg.logging_locale(), Locale::KoKr);
    }

    #[test]
    fn toml_roundtrip_default() {
        let cfg = LocaleConfig::default();
        let s = toml::to_string(&cfg).expect("serialize");
        let restored: LocaleConfig = toml::from_str(&s).expect("deserialize");
        assert_eq!(restored.client_java_edition, "auto");
        assert_eq!(restored.client_bedrock_edition, "auto");
        assert_eq!(restored.server_command, "auto");
        assert_eq!(restored.server_logging, "auto");
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
        assert_eq!(cfg.server_logging, "auto");
    }
}
