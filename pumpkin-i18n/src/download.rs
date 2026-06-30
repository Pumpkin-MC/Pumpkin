//! Runtime translation downloader.
//!
//! Downloads translation files from a remote mirror at server startup.
//! Falls back to compile-time embedded English translations on any failure.
//!
//! # Background loading
//! Per-player locale translations are loaded asynchronously via
//! [`ensure_locale_translations`] so that players can join immediately
//! while their language files download in the background.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::Duration;

use std::fmt::Write;

use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::locale::Locale;

/// Creates an HTTP agent with the given request timeout.
fn create_agent(timeout: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(timeout))
        .build()
        .into()
}

/// Default remote translation repository when no mirror URL is configured.
const DEFAULT_MIRROR: &str =
    "https://raw.githubusercontent.com/Q2297045667/Pumpkin/refactor_i18n/assets/translations";

/// Holds downloaded translations for a single locale, organised by namespace.
#[derive(Debug, Default)]
pub struct DownloadedTranslations {
    /// `pumpkin:` namespace entries.
    pub pumpkin: HashMap<String, String>,
    /// `java_minecraft:` namespace (Java Edition vanilla) entries.
    pub java: HashMap<String, String>,
    /// `bedrock_minecraft:` namespace (Bedrock Edition vanilla) entries.
    pub bedrock: HashMap<String, String>,
}

impl DownloadedTranslations {
    /// Returns `true` when at least one namespace has entries.
    #[must_use]
    pub fn has_any(&self) -> bool {
        !self.pumpkin.is_empty() || !self.java.is_empty() || !self.bedrock.is_empty()
    }
}

/// Configuration for the translation downloader.
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// Base URL of the mirror. Empty string means use the default mirror.
    pub mirror_url: String,
    /// Timeout for each individual HTTP request, in milliseconds.
    pub timeout_ms: u64,
    /// Skip SHA256 checksum verification for downloaded files.
    pub skip_checksum: bool,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            mirror_url: String::new(),
            timeout_ms: 10000,
            skip_checksum: false,
        }
    }
}

/// Attempts to download translation files for the given locale.
///
/// # Download targets
/// 1. `{mirror}/pumpkin/{code}.json` — pumpkin server translations
/// 2. `{mirror}/vanilla/{code}_java.json` — vanilla Java Edition translations
///
/// Bedrock Edition translations are **not** downloaded at runtime; only the
/// compile-time embedded `en_us` Bedrock strings are used.
///
/// Each file is fetched independently; partial failures are tolerated.
///
/// # Timeout
/// The configured timeout applies per request. If any request exceeds
/// the timeout, it is treated as a failure (the caller should fall back
/// to embedded English).
///
/// # Locale code
/// The locale code is derived from [`Locale::to_code`], e.g. `"en_us"`, `"zh_cn"`.
pub fn download_locale(config: &DownloadConfig, locale: Locale) -> DownloadedTranslations {
    let base_url = if config.mirror_url.is_empty() {
        DEFAULT_MIRROR
    } else {
        config.mirror_url.trim_end_matches('/')
    };

    let code = locale.to_code();
    let timeout = Duration::from_millis(config.timeout_ms);
    let skip_checksum = config.skip_checksum;

    let mut result = DownloadedTranslations::default();

    // 1. Pumpkin translations
    let pumpkin_url = format!("{base_url}/pumpkin/{code}.json");
    match fetch_json(&pumpkin_url, timeout, skip_checksum) {
        Ok(map) => {
            debug!("Downloaded pumpkin/{code}.json ({} entries)", map.len());
            result.pumpkin = map;
        }
        Err(e) => {
            warn!("Failed to download pumpkin/{code}.json: {e}");
        }
    }

    // 2. Vanilla Java Edition translations
    let java_url = format!("{base_url}/vanilla/{code}_java.json");
    match fetch_json(&java_url, timeout, skip_checksum) {
        Ok(map) => {
            debug!(
                "Downloaded vanilla/{code}_java.json ({} entries)",
                map.len()
            );
            result.java = map;
        }
        Err(e) => {
            warn!("Failed to download vanilla/{code}_java.json: {e}");
        }
    }

    // Bedrock Edition translations are compile-time embedded en_us only.
    // No runtime download for bedrock_minecraft namespace.

    result
}

/// Loads downloaded translations into the global translation store.
///
/// # Namespaces
/// * `pumpkin:` — pumpkin server translations
/// * `java_minecraft:` — vanilla Java Edition translations
///
/// Bedrock Edition translations are **not** loaded at runtime; the
/// compile-time embedded `en_us` Bedrock strings are always used.
///
/// This function calls [`crate::store::add_translation_file`] for each
/// namespace that has entries.
///
/// Emits a single consolidated log line summarising translation coverage.
/// Empty namespaces in non‑English locales produce a [`warn!`]; otherwise
/// the summary is logged at [`info!`] level.
pub fn load_downloaded(downloaded: &DownloadedTranslations, locale: Locale) {
    if !downloaded.pumpkin.is_empty() {
        let json = serde_json::to_string(&downloaded.pumpkin).unwrap();
        crate::store::add_translation_file("pumpkin", &json, locale);
    }

    if !downloaded.java.is_empty() {
        let json = serde_json::to_string(&downloaded.java).unwrap();
        crate::store::add_translation_file("java_minecraft", &json, locale);
    }

    // Bedrock Edition: compile-time embedded en_us only, no runtime loading.

    if !downloaded.has_any() {
        return;
    }

    let code = locale.to_code();
    let pumpkin_count = downloaded.pumpkin.len();
    let java_count = downloaded.java.len();

    // For non‑English locales, flag any empty namespace that will silently
    // fall back to English — users need to know this at a glance.
    if locale != Locale::EnUs && (pumpkin_count == 0 || java_count == 0) {
        let mut missing = Vec::with_capacity(2);
        if pumpkin_count == 0 {
            missing.push("server messages");
        }
        if java_count == 0 {
            missing.push("Java Edition vanilla strings");
        }
        warn!(
            "Translation coverage for {code}: pumpkin={pumpkin_count}, java={java_count} — {missing} will use English fallback",
            missing = missing.join(", "),
        );
    } else {
        info!("Loaded translations for {code} (pumpkin: {pumpkin_count}, java: {java_count})",);
    }
}

/// Builds the translation cache directory path for a given locale.
///
/// Returns `{cache_root}/{locale_code}/`.
fn translation_cache_dir(cache_root: &Path, locale: Locale) -> PathBuf {
    cache_root.join(locale.to_code())
}

/// Saves downloaded translations to disk under `{cache_root}/{locale_code}/`.
///
/// Creates the directory structure if it doesn't exist.
/// Each namespace is saved as a separate JSON file.
///
/// Bedrock Edition translations are not saved — only the compile-time
/// embedded `en_us` Bedrock strings are used.
///
/// # File layout
/// ```text
/// {cache_root}/en_us/pumpkin.json
/// {cache_root}/en_us/java_minecraft.json
/// ```
pub fn save_downloaded_translations(
    downloaded: &DownloadedTranslations,
    locale: Locale,
    cache_root: &Path,
) {
    let dir = translation_cache_dir(cache_root, locale);

    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!(
            "Failed to create translation cache directory {:?}: {e}",
            dir
        );
        return;
    }

    save_namespace_if_present(&dir, "pumpkin", &downloaded.pumpkin);
    save_namespace_if_present(&dir, "java_minecraft", &downloaded.java);
    // Bedrock Edition: compile-time embedded en_us only, not saved to disk.

    if downloaded.has_any() {
        info!(
            "Saved downloaded translations for {} to {:?}",
            locale.to_code(),
            dir
        );
    }
}

/// Write a single namespace's translation data to `{dir}/{file_name}.json`.
fn save_namespace_if_present(dir: &Path, file_name: &str, data: &HashMap<String, String>) {
    if data.is_empty() {
        return;
    }
    let path = dir.join(format!("{file_name}.json"));
    match serde_json::to_string_pretty(data) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, &json) {
                warn!("Failed to save {file_name} translations to {:?}: {e}", path);
            } else {
                debug!(
                    "Saved {file_name} translations to {:?} ({} entries)",
                    path,
                    data.len()
                );
            }
        }
        Err(e) => {
            warn!("Failed to serialize {file_name} translations: {e}");
        }
    }
}

/// Attempts to load cached translations from disk.
///
/// Looks for translation files under `{cache_root}/{locale_code}/`.
/// Returns `None` if the directory doesn't exist or no files are found.
///
/// Bedrock Edition translations are not cached on disk — only the
/// compile-time embedded `en_us` Bedrock strings are used.
///
/// # File layout
/// ```text
/// {cache_root}/en_us/pumpkin.json
/// {cache_root}/en_us/java_minecraft.json
/// ```
#[must_use]
pub fn load_cached_translations(
    locale: Locale,
    cache_root: &Path,
) -> Option<DownloadedTranslations> {
    let dir = translation_cache_dir(cache_root, locale);

    if !dir.exists() {
        return None;
    }

    let mut result = DownloadedTranslations::default();
    let mut found_any = false;

    found_any |= load_namespace_from_cache(&dir, "pumpkin", &mut result.pumpkin);
    found_any |= load_namespace_from_cache(&dir, "java_minecraft", &mut result.java);
    // Bedrock Edition: compile-time embedded en_us only, not loaded from cache.

    found_any.then(|| {
        info!(
            "Loaded cached translations for {} from {:?}",
            locale.to_code(),
            dir
        );
        result
    })
}

/// Try to load a single namespace's JSON file from the cache directory.
/// Returns `true` if valid data was loaded.
fn load_namespace_from_cache(
    dir: &Path,
    file_name: &str,
    dest: &mut HashMap<String, String>,
) -> bool {
    let path = dir.join(format!("{file_name}.json"));
    if !path.exists() {
        return false;
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<HashMap<String, String>>(&content) {
            Ok(map) if !map.is_empty() => {
                debug!(
                    "Loaded cached {file_name} translations from {:?} ({} entries)",
                    path,
                    map.len()
                );
                *dest = map;
                true
            }
            Ok(_) => {
                warn!("Cached {file_name} translation file is empty: {:?}", path);
                false
            }
            Err(e) => {
                warn!("Failed to parse cached {file_name} translations: {e}");
                false
            }
        },
        Err(e) => {
            warn!("Failed to read cached {file_name} translations: {e}");
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Background locale loader
// ---------------------------------------------------------------------------

/// Stores the download configuration and cache root for background locale loading.
/// Initialised once during server startup via [`init_translation_loader`].
static LOADER_STATE: OnceLock<(DownloadConfig, PathBuf)> = OnceLock::new();

/// Tracks locales that have already been loaded or are currently being loaded.
/// Prevents duplicate downloads for the same locale.
static LOADED_LOCALES: LazyLock<Mutex<HashSet<Locale>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Mark a locale as already loaded so that subsequent calls to
/// [`ensure_locale_translations`] skip it without redundant disk I/O.
///
/// Call this when the caller has already completed the download → save →
/// inject workflow outside of [`ensure_locale_translations`] (e.g. server
/// startup loading the global locale).
pub fn mark_locale_loaded(locale: Locale) {
    LOADED_LOCALES.lock().unwrap().insert(locale);
}

/// Initialise the translation loader with download configuration and cache root.
///
/// Must be called once during server startup, before any background locale
/// loading is triggered. The config is used by [`ensure_locale_translations`]
/// to download missing translations on demand.
///
/// # Panics
/// Never panics. Subsequent calls log a warning and are ignored.
pub fn init_translation_loader(config: DownloadConfig, cache_root: PathBuf) {
    match LOADER_STATE.set((config, cache_root)) {
        Ok(()) => {}
        Err(_) => {
            warn!(
                "init_translation_loader called more than once — ignoring duplicate initialisation"
            );
        }
    }
}

/// Ensure translations are loaded for the given locale.
///
/// # Behaviour
/// 1. **`EnUs`** — no-op (embedded at compile time).
/// 2. **Already loaded** — no-op (deduplicated via internal tracking set).
/// 3. **Complete disk cache** — loads both namespaces and returns.
/// 4. **Partial disk cache** — loads what exists, then downloads the full
///    set to fill gaps.
/// 5. **No cache** — downloads from the configured mirror, saves to disk,
///    and injects into the engine.
///
/// Bedrock Edition translations are **not** downloaded or cached; only the
/// compile-time embedded `en_us` Bedrock strings are used.
///
/// # Thread safety
/// Safe to call from multiple threads. The tracking set ensures the same
/// locale is only processed once, even under concurrent calls.
///
/// # Failure handling
/// Errors during download or disk I/O are logged at [`warn!`] level.
/// The function never panics; callers can treat it as fire-and-forget.
pub fn ensure_locale_translations(locale: Locale) {
    // English is embedded at compile time — nothing to load
    if locale == Locale::EnUs {
        return;
    }

    // Check if already loaded or being loaded
    {
        let mut loaded = LOADED_LOCALES.lock().unwrap();
        if !loaded.insert(locale) {
            return; // Already handled
        }
    }

    let Some((config, cache_root)) = LOADER_STATE.get() else {
        warn!(
            "Translation loader not initialised — cannot load translations for {}",
            locale.to_code()
        );
        return;
    };

    // 1. Try disk cache first
    let partial_cache = if let Some(cached) = load_cached_translations(locale, cache_root) {
        let complete = !cached.pumpkin.is_empty() && !cached.java.is_empty();
        if complete {
            // Both namespaces present — no download needed
            load_downloaded(&cached, locale);
            return;
        }
        // Partial cache: load what we have now, then download the full set
        load_downloaded(&cached, locale);
        true
    } else {
        false
    };

    // 2. Download full set from remote mirror
    let downloaded = download_locale(config, locale);

    // 3. Save to disk for future runs
    if downloaded.has_any() {
        save_downloaded_translations(&downloaded, locale, cache_root);

        // 4. Inject into the global engine (overwrites partial cache data)
        load_downloaded(&downloaded, locale);
    } else if !partial_cache {
        warn!(
            "No translations available for {} — using English fallback",
            locale.to_code()
        );
    }
    // If partial_cache is true and download failed, we already loaded
    // the partial cache above. The missing namespaces will use EnUs fallback.
}

// ---------------------------------------------------------------------------
// SHA256 checksum verification
// ---------------------------------------------------------------------------

/// Downloads the SHA256 checksum file for a translation file.
///
/// The checksum file is expected at `{data_url}.sha256`.
/// Accepts standard `sha256sum` format (`"hash  filename"`) or bare hex hash.
fn fetch_sha256(data_url: &str, timeout: Duration) -> Result<String, String> {
    let checksum_url = format!("{data_url}.sha256");
    let agent = create_agent(timeout);
    let response = agent
        .get(&checksum_url)
        .call()
        .map_err(|e| format!("checksum download failed: {e}"))?;

    let body = response
        .into_body()
        .read_to_string()
        .map_err(|e| format!("checksum read failed: {e}"))?;

    // Parse first line: either "hash  filename" or bare hash
    let hash = body
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .next()
        .unwrap_or("");

    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid checksum format".to_string());
    }

    Ok(hash.to_ascii_lowercase())
}

/// Verifies that `data` matches the expected SHA256 hex digest.
fn verify_sha256(data: &[u8], expected_hex: &str) -> Result<(), String> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let actual = hasher
        .finalize()
        .iter()
        .fold(String::with_capacity(64), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        });
    if actual != expected_hex {
        return Err(format!(
            "checksum mismatch: expected {expected_hex}, got {actual}"
        ));
    }
    Ok(())
}

/// Attempts to verify a downloaded payload against its `.sha256` file.
///
/// If the checksum file is unavailable (404, timeout, etc.), a warning is
/// logged but the data is **accepted** — this maintains backward compatibility
/// with mirrors that don't host checksum files.
///
/// If the checksum file is available but the hash does NOT match, the data
/// is **rejected** and an error is returned.
fn try_verify_checksum(data_url: &str, data: &[u8], timeout: Duration) -> Result<(), String> {
    match fetch_sha256(data_url, timeout) {
        Ok(expected) => verify_sha256(data, &expected),
        Err(e) => {
            warn!("Skipping checksum verification for {data_url}: {e}");
            Ok(()) // Degrade gracefully — accept data without checksum
        }
    }
}

// ---------------------------------------------------------------------------
// Internal fetchers
// ---------------------------------------------------------------------------

/// Fetch and parse a JSON translation file from the given URL.
///
/// After downloading, the data is verified against a `.sha256` checksum file
/// if one is available on the mirror — unless `skip_checksum` is `true`.
fn fetch_json(
    url: &str,
    timeout: Duration,
    skip_checksum: bool,
) -> Result<HashMap<String, String>, String> {
    let agent = create_agent(timeout);
    let response = agent
        .get(url)
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let body = response
        .into_body()
        .read_to_string()
        .map_err(|e| format!("failed to read response body: {e}"))?;

    // Verify checksum if available; reject on mismatch
    if !skip_checksum {
        try_verify_checksum(url, body.as_bytes(), timeout)?;
    }

    let map: HashMap<String, String> =
        serde_json::from_str(&body).map_err(|e| format!("failed to parse JSON: {e}"))?;

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_config_defaults() {
        let config = DownloadConfig::default();
        assert!(config.mirror_url.is_empty());
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn downloaded_translations_has_any() {
        let mut dt = DownloadedTranslations::default();
        assert!(!dt.has_any());

        dt.pumpkin.insert("key".to_string(), "value".to_string());
        assert!(dt.has_any());
    }

    #[test]
    fn verify_sha256_matches_correctly() {
        let data = b"hello world";
        // SHA256 of "hello world"
        let expected = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert!(super::verify_sha256(data, expected).is_ok());
    }

    #[test]
    fn verify_sha256_detects_mismatch() {
        let data = b"hello world";
        let wrong = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(super::verify_sha256(data, wrong).is_err());
    }
}
