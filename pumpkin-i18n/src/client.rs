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
static PLAYER_CACHE: LazyLock<PlayerCache> =
    LazyLock::new(|| DashMap::with_hasher(BuildHasherDefault::default()));

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
    PLAYER_CACHE
        .get(uuid)
        .map_or(Locale::EnUs, |entry| *entry.value())
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
    let source = if config_value == "auto" {
        player_locale
    } else {
        config_value
    };
    crate::parse_locale_value(source)
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// Returns the locale string in lowercase underscore format for logging.
///
/// # Arguments
/// * `locale` — The locale to format.
///
/// # Returns
/// A string like `"en_us"`, `"zh_cn"`, etc.
#[must_use]
pub fn locale_to_log_string(locale: Locale) -> String {
    let raw = format!("{locale:?}");
    // Insert underscores before uppercase letters, then lowercase
    raw.chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i > 0 && c.is_uppercase() {
                vec!['_', c.to_ascii_lowercase()]
            } else {
                vec![c.to_ascii_lowercase()]
            }
        })
        .collect()
}
