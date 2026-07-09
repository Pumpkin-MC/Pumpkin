pub mod client;
pub mod download;
pub mod engine;
pub mod locale;
pub mod server;
pub mod store;
pub mod token;

pub use client::{
    player_locale, remove_player_locale, resolve_client_locale, set_player_locale,
    try_player_locale,
};
pub use download::{
    DownloadConfig, DownloadedTranslations, download_locale, ensure_locale_translations,
    init_translation_loader, load_cached_translations, load_downloaded, mark_locale_loaded,
    save_downloaded_translations, save_checksums,
};
pub use engine::{ResolvedTranslation, TranslationEngine, format_tokens};
pub use locale::Locale;
pub use server::{
    detect_system_locale, resolve_server_locale, server_global_locale, set_server_global_locale,
};
pub use store::{
    add_translation, add_translation_file, format_translation, get_translation,
    resolve_translation, translation_engine,
};
pub use token::{Token, placeholder_ranges, precompile};

use std::str::FromStr;

/// Namespace prefix for all pumpkin server translation keys.
pub const PUMPKIN_NAMESPACE: &str = "pumpkin";

/// Namespace for vanilla Java Edition translation keys.
pub const JAVA_NAMESPACE: &str = "java_minecraft";

/// Namespace for vanilla Bedrock Edition translation keys.
pub const BEDROCK_NAMESPACE: &str = "bedrock_minecraft";

/// Build a fully qualified translation key under the pumpkin namespace.
///
/// Equivalent to calling [`namespaced_key`] with `PUMPKIN_NAMESPACE`.
#[must_use]
#[inline]
pub fn pumpkin_translation_key(key: &str) -> String {
    namespaced_key(PUMPKIN_NAMESPACE, key)
}

/// Build a namespaced translation key in the form `"namespace:key"`.
///
/// The resulting key is **not** lowercased — callers that need
/// case-insensitive lookups should lower-case the result themselves
/// (e.g. via [`str::to_ascii_lowercase`]).
#[must_use]
#[inline]
pub fn namespaced_key(namespace: &str, key: &str) -> String {
    let mut out = String::with_capacity(namespace.len() + key.len() + 1);
    out.push_str(namespace);
    out.push(':');
    out.push_str(key);
    out
}

/// Parse a locale identifier string without unnecessary allocations.
///
/// Normalises hyphens to underscores only when needed and uses
/// ASCII-only lowercasing. Returns [`Locale::EnUs`] on failure.
pub(crate) fn parse_locale_value(raw: &str) -> Locale {
    Locale::from_str(raw).unwrap_or(Locale::EnUs)
}

/// A character range representing a substitution placeholder within a translation string.
///
/// The range is inclusive and corresponds to the full placeholder span
/// (for example `%s` or `%1$s`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionRange {
    /// Start byte index (inclusive).
    pub start: usize,
    /// End byte index (inclusive).
    pub end: usize,
}
impl SubstitutionRange {
    /// Returns the length of the range.
    #[must_use]
    pub const fn len(&self) -> usize {
        (self.end - self.start) + 1
    }
    /// Returns `true` when `end < start` — an inclusive range cannot
    /// be empty otherwise (an inclusive range always covers at least
    /// one character when `start <= end`).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.end < self.start
    }
}
