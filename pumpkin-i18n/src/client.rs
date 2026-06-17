//! Per-player client locale resolution.
//!
//! Each connected player reports a language string (e.g. `"zh_cn"`) during
//! login.  Combined with the server's per-edition `[locale]` setting, this
//! module resolves the effective [`Locale`] for Pumpkin custom translations
//! sent to that player.
//!
//! A simple in-memory cache maps player identifiers to their resolved
//! locale so that callers do not need to re-resolve on every message.
//! Callers **must** call [`remove_player_locale`] when the player
//! disconnects to avoid leaking entries.

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

use crate::Locale;

// ---------------------------------------------------------------------------
// Global per-player cache
// ---------------------------------------------------------------------------

/// In-memory cache: player-id → resolved [`Locale`].
static PLAYER_CACHE: std::sync::LazyLock<Mutex<HashMap<String, Locale>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Stores (or updates) the locale for the given player.
///
/// Returns the resolved [`Locale`] so the caller can use it immediately
/// without a second lookup.
pub fn set_player_locale(player_id: &str, player_lang: &str, edition_setting: &str) -> Locale {
    let locale = resolve_client_locale(player_lang, edition_setting);
    PLAYER_CACHE
        .lock()
        .unwrap()
        .insert(player_id.to_owned(), locale);
    locale
}

/// Returns the cached locale for a player, if any.
#[must_use]
pub fn get_player_locale(player_id: &str) -> Option<Locale> {
    PLAYER_CACHE.lock().unwrap().get(player_id).copied()
}

/// Removes a player from the cache (call on disconnect).
pub fn remove_player_locale(player_id: &str) {
    PLAYER_CACHE.lock().unwrap().remove(player_id);
}

// ---------------------------------------------------------------------------
// Resolution logic
// ---------------------------------------------------------------------------

/// Resolves the effective locale for a client.
///
/// # Arguments
/// * `player_lang` — The raw language string reported by the client
///   (`"zh_cn"`, `"en_US"`, …).  May be empty.
/// * `edition_setting` — The server's `[locale].client_*_edition` value.
///   `"auto"` means "use the player's own language"; anything else forces
///   that specific locale on all players.
///
/// # Returns
/// The resolved [`Locale`].  Falls back to [`Locale::EnUs`] when neither
/// the player language nor the edition setting produces a valid locale.
#[must_use]
pub fn resolve_client_locale(player_lang: &str, edition_setting: &str) -> Locale {
    // 1. Config forces a specific locale.
    if !edition_setting.eq_ignore_ascii_case("auto") {
        return Locale::from_str(edition_setting).unwrap_or(Locale::EnUs);
    }

    // 2. "auto" — use the player's own language.
    if player_lang.is_empty() {
        return Locale::EnUs;
    }

    // Both "en_US" (Bedrock) and "en_us" (Java) are accepted by the
    // case-insensitive FromStr impl.
    Locale::from_str(player_lang).unwrap_or(Locale::EnUs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- resolve_client_locale -------------------------------------------------

    #[test]
    fn auto_uses_player_lang() {
        assert_eq!(resolve_client_locale("zh_cn", "auto"), Locale::ZhCn);
        assert_eq!(resolve_client_locale("de_de", "auto"), Locale::DeDe);
    }

    #[test]
    fn bedrock_case_normalised() {
        assert_eq!(resolve_client_locale("en_US", "auto"), Locale::EnUs);
        assert_eq!(resolve_client_locale("zh_CN", "auto"), Locale::ZhCn);
    }

    #[test]
    fn forced_locale_overrides_player() {
        assert_eq!(resolve_client_locale("zh_cn", "de_de"), Locale::DeDe);
    }

    #[test]
    fn empty_lang_falls_back() {
        assert_eq!(resolve_client_locale("", "auto"), Locale::EnUs);
    }

    #[test]
    fn invalid_falls_back() {
        assert_eq!(resolve_client_locale("not_a_locale", "auto"), Locale::EnUs);
    }

    // -- cache ----------------------------------------------------------------

    #[test]
    fn cache_set_and_get() {
        let id = "test-player-1";
        set_player_locale(id, "zh_cn", "auto");
        assert_eq!(get_player_locale(id), Some(Locale::ZhCn));
    }

    #[test]
    fn cache_remove() {
        let id = "test-player-2";
        set_player_locale(id, "de_de", "auto");
        remove_player_locale(id);
        assert_eq!(get_player_locale(id), None);
    }

    #[test]
    fn cache_update() {
        let id = "test-player-3";
        set_player_locale(id, "fr_fr", "auto");
        set_player_locale(id, "ja_jp", "auto");
        assert_eq!(get_player_locale(id), Some(Locale::JaJp));
    }
}
