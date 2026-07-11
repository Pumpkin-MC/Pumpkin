use std::hash::BuildHasherDefault;
use std::sync::LazyLock;

use dashmap::DashMap;
use xxhash_rust::xxh64::Xxh64;

use crate::locale::Locale;

// ---------------------------------------------------------------------------
// Player locale cache (UUID → Locale)
// ---------------------------------------------------------------------------

type PlayerCache = DashMap<String, Locale, BuildHasherDefault<Xxh64>>;

/// Global in‑memory cache mapping player UUIDs to their resolved locale.
///
/// Populated on login, read during translation lookups, and cleaned on
/// disconnect. Uses [`DashMap`] with XXH64 hashing for lock‑free concurrent
/// reads.
///
/// An upper-bound guard clears the cache when it exceeds [`MAX_PLAYER_CACHE_SIZE`]
/// entries so that a missing [`remove_player_locale`] call (e.g. disconnect-
/// handler race, crash) cannot cause unbounded growth over very long uptimes.
static PLAYER_CACHE: LazyLock<PlayerCache> =
    LazyLock::new(|| DashMap::with_hasher(BuildHasherDefault::default()));

/// Maximum number of cached player locales before the cache is flushed.
/// At ~60 bytes per entry (36-byte UUID + 1-byte Locale + DashMap overhead),
/// 100k entries ≈ 6 MB — well under the memory budget for even large networks.
const MAX_PLAYER_CACHE_SIZE: usize = 100_000;

/// Resolve and cache a player's locale on login.
///
/// # Arguments
/// * `uuid` — The player's UUID string (e.g. `"550e8400-e29b-41d4-a716-446655440000"`).
/// * `player_reported_locale` — The locale string sent by the client.
/// * `config_value` — The server's locale config value (`"auto"` or a specific code).
///
/// # Returns
/// The resolved [`Locale`], which has also been stored in [`PLAYER_CACHE`].
pub fn set_player_locale(uuid: &str, player_reported_locale: &str, config_value: &str) -> Locale {
    let locale = resolve_client_locale(player_reported_locale, config_value);

    // Guard against unbounded growth: if the cache exceeds the cap (e.g.
    // because remove_player_locale was never called for many players due to
    // disconnect-handler races), clear the stale entries in one shot.
    if PLAYER_CACHE.len() >= MAX_PLAYER_CACHE_SIZE {
        PLAYER_CACHE.clear();
    }

    PLAYER_CACHE.insert(uuid.to_owned(), locale);
    locale
}

/// Retrieve a player's cached locale.
///
/// Falls back to [`Locale::EnUs`] when the UUID is not found in the cache.
///
/// # Arguments
/// * `uuid` — The player's UUID string.
///
/// # Returns
/// The cached [`Locale`], or [`Locale::EnUs`] on cache miss.
#[must_use]
pub fn player_locale(uuid: &str) -> Locale {
    try_player_locale(uuid).unwrap_or(Locale::EnUs)
}

/// Retrieve a player's cached locale, returning [`None`] on cache miss.
#[must_use]
pub fn try_player_locale(uuid: &str) -> Option<Locale> {
    PLAYER_CACHE.get(uuid).map(|entry| *entry.value())
}

/// Remove a player from the locale cache on disconnect.
///
/// # Arguments
/// * `uuid` — The player's UUID string.
pub fn remove_player_locale(uuid: &str) {
    PLAYER_CACHE.remove(uuid);
}

// ---------------------------------------------------------------------------
// Client locale resolution
// ---------------------------------------------------------------------------

/// Resolves the client locale for a player based on the configuration value
/// and the locale reported by the player's client.
///
/// # Arguments
/// * `player_locale` — The locale string reported by the client (e.g. `"en_us"`, `"zh_cn"`).
/// * `config_value` — The locale configuration value, either `"auto"` or a specific locale code.
///
/// # Returns
/// The resolved [`Locale`]. If `config_value` is `"auto"`, returns the player's locale.
/// Otherwise overrides with the configured locale.
#[must_use]
pub fn resolve_client_locale(player_locale: &str, config_value: &str) -> Locale {
    let source = if config_value.eq_ignore_ascii_case("auto") {
        player_locale
    } else {
        config_value
    };
    crate::parse_locale_value(source)
}
