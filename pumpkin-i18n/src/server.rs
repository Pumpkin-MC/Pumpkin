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
