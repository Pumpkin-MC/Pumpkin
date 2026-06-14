//! Translation utilities for plugins.
//!
//! Re-exports the underlying WIT `translate` and `load_translations` for
//! backward compatibility, and provides a higher-level `translate_text`
//! that delegates placeholder substitution to the host.

pub use crate::wit::pumpkin::plugin::i18n::{load_translations, translate};

use crate::wit::pumpkin::plugin::common;

/// Resolves a translation key and substitutes placeholders.
///
/// Supported formats (`%s`, `%N$s`, `{0}`) are handled server-side via
/// [`get_translation_text`](pumpkin_util::translation::get_translation_text).
///
/// If `with` is empty the raw translation is returned unchanged.
/// When `with` has fewer items than placeholders the last value is reused.
#[must_use]
pub fn translate_text(key: &str, locale: common::Locale, with: &[String]) -> String {
    if with.is_empty() {
        return translate(key, locale);
    }
    crate::wit::pumpkin::plugin::i18n::translate_text(key, locale, with)
}

/// Loads custom translations for the given locale from a flat JSON map.
///
/// Convenience wrapper around `load_translations`.
pub fn load(namespace: &str, json: &str, locale: common::Locale) {
    load_translations(namespace, json, locale);
}
