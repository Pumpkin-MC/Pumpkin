/// TODO List
/// - Open a public translation system, maybe a Crowdin like Minecraft?
/// - Add support for translations on commands descriptions
/// - Integrate custom translations with the plugins API
pub mod client;
pub mod engine;
pub mod locale;
pub mod server;
pub mod store;
pub mod token;

pub use client::{
    locale_to_log_string, player_locale, remove_player_locale, resolve_client_locale,
    set_player_locale, try_player_locale,
};
pub use engine::{ResolvedTranslation, TranslationEngine, format_tokens};
pub use locale::Locale;
pub use server::{
    detect_system_locale, resolve_server_locale, server_command_locale, server_global_locale,
    set_server_command_locale, set_server_global_locale,
};
pub use store::{
    TRANSLATIONS, add_translation, add_translation_file, format_translation, get_translation,
    resolve_translation, translation_engine,
};
pub use token::{Token, placeholder_ranges, precompile};

use std::str::FromStr;

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
    /// Returns `true` if the range contains no characters.
    ///
    /// A range is considered empty when `start == end`.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}
