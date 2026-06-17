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
///
/// # Panics
///
/// Panics if the player cache [`Mutex`] is poisoned.
pub fn set_player_locale(player_id: &str, player_lang: &str, edition_setting: &str) -> Locale {
    let locale = resolve_client_locale(player_lang, edition_setting);
    PLAYER_CACHE
        .lock()
        .unwrap()
        .insert(player_id.to_owned(), locale);
    locale
}

/// Returns the cached locale for a player, if any.
///
/// # Panics
///
/// Panics if the player cache [`Mutex`] is poisoned.
#[must_use]
pub fn get_player_locale(player_id: &str) -> Option<Locale> {
    PLAYER_CACHE.lock().unwrap().get(player_id).copied()
}

/// Removes a player from the cache (call on disconnect).
///
/// # Panics
///
/// Panics if the player cache [`Mutex`] is poisoned.
pub fn remove_player_locale(player_id: &str) {
    PLAYER_CACHE.lock().unwrap().remove(player_id);
}

// ---------------------------------------------------------------------------
// Convenience: setup / lookup / teardown for editions
// ---------------------------------------------------------------------------

/// Resolves the locale for a Java Edition player using the server
/// `[locale].client_java_edition` setting.
#[must_use]
pub fn setup_java_player(player_id: &str, player_lang: &str, edition_setting: &str) -> Locale {
    set_player_locale(player_id, player_lang, edition_setting)
}

/// Resolves the locale for a Bedrock Edition player using the server
/// `[locale].client_bedrock_edition` setting.
#[must_use]
pub fn setup_bedrock_player(player_id: &str, player_lang: &str, edition_setting: &str) -> Locale {
    set_player_locale(player_id, player_lang, edition_setting)
}

/// Returns the cached locale for a player (defaults to [`Locale::EnUs`]).
#[must_use]
pub fn player_locale(player_id: &str) -> Locale {
    get_player_locale(player_id).unwrap_or(Locale::EnUs)
}

/// Removes a player from the cache (call on disconnect).
pub fn teardown_player(player_id: &str) {
    remove_player_locale(player_id);
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
// Validation (called by pumpkin-config at startup)
// ---------------------------------------------------------------------------

/// Logs a warning for every client edition field that is neither `"auto"`,
/// empty, nor a recognised locale identifier.
pub fn validate_locale_config(java_setting: &str, bedrock_setting: &str) {
    for (label, value) in [
        ("client_java_edition", java_setting),
        ("client_bedrock_edition", bedrock_setting),
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

    // -- full integration flow ------------------------------------------------

    #[test]
    fn full_flow_two_players_join_execute_command_disconnect() {
        // ── 1. 服务器配置 ─────────────────────────────────────────────
        let config_java = "auto";
        let config_bedrock = "auto";

        // ── 2. 玩家 A（Java 版, 英语）加入 ────────────────────────────
        let java_id = "550e8400-e29b-41d4-a716-446655440000"; // uuid
        let java_lang = "en_us";
        let locale_a = setup_java_player(java_id, java_lang, config_java);
        assert_eq!(locale_a, Locale::EnUs);
        assert_eq!(player_locale(java_id), Locale::EnUs);

        // ── 3. 玩家 B（Bedrock 版, 简体中文）加入 ──────────────────────
        let bedrock_id = "660e8400-e29b-41d4-a716-446655440001";
        let bedrock_lang = "zh_CN"; // Bedrock 大写
        let locale_b = setup_bedrock_player(bedrock_id, bedrock_lang, config_bedrock);
        assert_eq!(locale_b, Locale::ZhCn);
        assert_eq!(player_locale(bedrock_id), Locale::ZhCn);

        // ── 4. 两个玩家执行 /pumpkin 命令 ─────────────────────────────
        // 用 get_translation 模拟 TextComponent::custom 的服务器端解析

        let key = "pumpkin:commands.pumpkin.description";

        let msg_a = crate::get_translation(key, player_locale(java_id));
        let msg_b = crate::get_translation(key, player_locale(bedrock_id));

        // 玩家 A 收到英文
        assert!(
            msg_a.contains("Empowering everyone"),
            "Player A should get English, got: {msg_a}"
        );

        // 玩家 B 收到简体中文
        assert!(
            msg_b.contains("让每个人都能搭建"),
            "Player B should get Chinese, got: {msg_b}"
        );

        // 同一个 key，不同语言 → 不同内容
        assert_ne!(msg_a, msg_b, "Same key different languages must differ");

        // ── 5. 玩家 B 断开连接 ────────────────────────────────────────
        teardown_player(bedrock_id);
        assert_eq!(get_player_locale(bedrock_id), None);

        // 玩家 A 仍然在缓存中
        assert_eq!(player_locale(java_id), Locale::EnUs);

        // ── 6. 玩家 A 断开连接 ────────────────────────────────────────
        teardown_player(java_id);
        assert_eq!(get_player_locale(java_id), None);
    }

    #[test]
    fn forced_edition_override_flow() {
        // 服务器强制 Java 版使用德语
        let forced_de = "de_de";
        let player_id = "770e8400-e29b-41d4-a716-446655440002";

        // 玩家说中文，但服务器强制德语
        let locale = setup_java_player(player_id, "zh_cn", forced_de);
        assert_eq!(locale, Locale::DeDe);

        let key = "pumpkin:commands.pumpkin.description";
        let msg = crate::get_translation(key, player_locale(player_id));

        assert!(
            msg.contains("Ermöglicht es jedem"),
            "Should get German (forced by config), got: {msg}"
        );

        teardown_player(player_id);
    }

    #[test]
    fn player_unknown_language_falls_back_to_english() {
        let player_id = "880e8400-e29b-41d4-a716-446655440003";
        let locale = setup_java_player(player_id, "xx_xx", "auto");
        assert_eq!(locale, Locale::EnUs);

        let key = "pumpkin:commands.pumpkin.description";
        let msg = crate::get_translation(key, player_locale(player_id));
        assert!(
            msg.contains("Empowering everyone"),
            "Unknown lang should fall back to English"
        );

        teardown_player(player_id);
    }
}
