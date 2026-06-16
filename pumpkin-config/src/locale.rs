use std::str::FromStr;

use pumpkin_i18n::Locale;
use serde::{Deserialize, Serialize};

/// Configuration for server-side translation / locale behaviour.
///
/// Controls which language is used for **logging output** and for
/// **command feedback** independently. Set either to `"auto"` to
/// auto-detect from the system, or to a specific locale identifier
/// (e.g. `"zh_cn"`, `"de_de"`).
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct LocaleConfig {
    /// Locale for server log messages.
    pub logging: String,
    /// Locale for command feedback sent to players and the console.
    pub command: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            logging: String::from("auto"),
            command: String::from("auto"),
        }
    }
}

impl LocaleConfig {
    /// Resolves the configured logging locale.
    #[must_use]
    pub fn logging_locale(&self) -> Locale {
        pumpkin_i18n::resolve_locale(&self.logging)
    }

    /// Resolves the configured command feedback locale.
    #[must_use]
    pub fn command_locale(&self) -> Locale {
        pumpkin_i18n::resolve_locale(&self.command)
    }

    /// Validates the locale configuration.
    ///
    /// Logs a warning for every field whose value is neither `"auto"` nor a
    /// recognised locale identifier. The runtime falls back to [`Locale::EnUs`]
    /// for those fields.
    pub fn validate(&self) {
        for (label, value) in [("logging", &self.logging), ("command", &self.command)] {
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
        assert_eq!(cfg.logging, "auto");
        assert_eq!(cfg.command, "auto");
    }

    #[test]
    fn explicit_locale_resolves() {
        let cfg = LocaleConfig {
            logging: "zh_cn".into(),
            command: "de_de".into(),
        };
        assert_eq!(cfg.logging_locale(), Locale::ZhCn);
        assert_eq!(cfg.command_locale(), Locale::DeDe);
    }

    #[test]
    fn invalid_locale_falls_back_to_en_us() {
        let cfg = LocaleConfig {
            logging: "not_a_real_locale".into(),
            command: "".into(),
        };
        assert_eq!(cfg.logging_locale(), Locale::EnUs);
        assert_eq!(cfg.command_locale(), Locale::EnUs);
    }

    #[test]
    fn toml_roundtrip_default() {
        let cfg = LocaleConfig::default();
        let toml_str = toml::to_string(&cfg).expect("serialize");
        let restored: LocaleConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(restored.logging, "auto");
        assert_eq!(restored.command, "auto");
    }

    #[test]
    fn toml_roundtrip_custom() {
        let toml_str = "logging = \"zh_cn\"\ncommand = \"de_de\"\n";
        let cfg: LocaleConfig = toml::from_str(toml_str).expect("deserialize");
        assert_eq!(cfg.logging, "zh_cn");
        assert_eq!(cfg.command, "de_de");
    }

    #[test]
    fn toml_missing_field_uses_default() {
        let cfg: LocaleConfig = toml::from_str("logging = \"fr_fr\"\n").expect("deserialize");
        assert_eq!(cfg.logging, "fr_fr");
        assert_eq!(cfg.command, "auto");
    }

    #[test]
    fn toml_empty_uses_defaults() {
        let cfg: LocaleConfig = toml::from_str("").expect("deserialize");
        assert_eq!(cfg.logging, "auto");
        assert_eq!(cfg.command, "auto");
    }
}
