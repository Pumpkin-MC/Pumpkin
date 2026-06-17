//! Server-side locale resolution for command feedback and log output.
//!
//! Resolves a configuration value (`"auto"` or an explicit locale) into a
//! concrete [`Locale`].  When `"auto"` is used the system locale is detected
//! via [`crate::detect_system_locale`].

use std::str::FromStr;

use crate::Locale;

/// Resolves a server-side locale from its configuration value.
///
/// # Arguments
/// * `setting` — The configuration string (`"auto"` or a locale identifier).
///
/// # Returns
/// The resolved [`Locale`].  Falls back to [`Locale::EnUs`] on failure.
#[must_use]
pub fn resolve_server_locale(setting: &str) -> Locale {
    if setting.eq_ignore_ascii_case("auto") {
        return crate::detect_system_locale();
    }
    Locale::from_str(setting).unwrap_or(Locale::EnUs)
}

/// Resolves the locale for command feedback from the `[locale].server_command` setting.
#[must_use]
pub fn command_locale(setting: &str) -> Locale {
    resolve_server_locale(setting)
}

/// Resolves the locale for log output from the `[locale].server_logging` setting.
#[must_use]
pub fn logging_locale(setting: &str) -> Locale {
    resolve_server_locale(setting)
}

/// Logs a warning for every server-side field that is neither `"auto"`,
/// empty, nor a recognised locale identifier.
pub fn validate_locale_config(command_setting: &str, logging_setting: &str) {
    for (label, value) in [
        ("server_command", command_setting),
        ("server_logging", logging_setting),
    ] {
        if value.eq_ignore_ascii_case("auto") || value.is_empty() {
            continue;
        }
        if Locale::from_str(value).is_err() {
            tracing::warn!(
                "[locale].{label} = \"{value}\" is not a recognised locale – \
                 falling back to English (en_us)"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_detects_system_locale() {
        let _ = resolve_server_locale("auto"); // must not panic
    }

    #[test]
    fn explicit_locale() {
        assert_eq!(resolve_server_locale("zh_cn"), Locale::ZhCn);
        assert_eq!(resolve_server_locale("de_de"), Locale::DeDe);
    }

    #[test]
    fn invalid_falls_back() {
        assert_eq!(resolve_server_locale("invalid"), Locale::EnUs);
    }
}
