use pumpkin_i18n::{
    Locale, PUMPKIN_NAMESPACE, format_translation, get_translation, pumpkin_translation_key,
    server_global_locale,
};

use crate::text::TextComponent;

// ---------------------------------------------------------------------------
// Explicit‑locale helpers (used by the command module and other callers that
// operate on a per‑sender locale).
// ---------------------------------------------------------------------------

/// Translate a pumpkin‑namespaced key for a specific locale (no formatting).
#[must_use]
pub fn translate_plain(key: &str, locale: Locale) -> String {
    get_translation(&pumpkin_translation_key(key), locale)
}

/// Translate a pumpkin‑namespaced key for a specific locale with format args.
#[must_use]
pub fn translate_format(key: &str, locale: Locale, args: &[String]) -> String {
    format_translation(&pumpkin_translation_key(key), locale, args)
}

// ---------------------------------------------------------------------------
// Server‑global‑locale convenience wrappers (log / console output).
// ---------------------------------------------------------------------------

#[must_use]
pub fn localized_log(key: &str) -> String {
    translate_plain(key, server_global_locale())
}

#[must_use]
pub fn localized_log_format(key: &str, args: &[String]) -> String {
    translate_format(key, server_global_locale(), args)
}

#[must_use]
pub fn localized_text<W>(key: &'static str, with: W) -> TextComponent
where
    W: Into<Vec<TextComponent>>,
{
    TextComponent::custom(PUMPKIN_NAMESPACE, key, server_global_locale(), with)
}
