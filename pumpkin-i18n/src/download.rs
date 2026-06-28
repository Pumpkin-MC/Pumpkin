//! Runtime translation downloader.
//!
//! Downloads translation files from a remote mirror at server startup.
//! Falls back to compile-time embedded English translations on any failure.

use std::collections::HashMap;
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
    /// `minecraft:` namespace (Java Edition vanilla) entries.
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
            timeout_ms: 1000,
            skip_checksum: false,
        }
    }
}

/// Attempts to download translation files for the given locale.
///
/// # Download targets
/// 1. `{mirror}/pumpkin/{code}.json` — pumpkin server translations
/// 2. `{mirror}/vanilla/{code}_java.json` — vanilla Java Edition translations
/// 3. `{mirror}/vanilla/{code}_bedrock.lang` — vanilla Bedrock Edition translations
///
/// Each file is fetched independently; partial failures are tolerated.
/// A download is considered fully successful only if **all three** files
/// are retrieved and parsed.
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

    // 3. Vanilla Bedrock Edition translations
    let bedrock_url = format!("{base_url}/vanilla/{code}_bedrock.lang");
    match fetch_bedrock_lang(&bedrock_url, timeout, skip_checksum) {
        Ok(map) => {
            debug!(
                "Downloaded vanilla/{code}_bedrock.lang ({} entries)",
                map.len()
            );
            result.bedrock = map;
        }
        Err(e) => {
            warn!("Failed to download vanilla/{code}_bedrock.lang: {e}");
        }
    }

    result
}

/// Loads downloaded translations into the global translation store.
///
/// # Namespaces
/// * `pumpkin:` — pumpkin server translations
/// * `minecraft:` — vanilla Java Edition translations
/// * `bedrock_minecraft:` — vanilla Bedrock Edition translations
///
/// This function calls [`crate::store::add_translation_file`] for each
/// namespace that has entries.
pub fn load_downloaded(downloaded: &DownloadedTranslations, locale: Locale) {
    if !downloaded.pumpkin.is_empty() {
        let json = serde_json::to_string(&downloaded.pumpkin).unwrap();
        crate::store::add_translation_file("pumpkin", &json, locale);
    }

    if !downloaded.java.is_empty() {
        let json = serde_json::to_string(&downloaded.java).unwrap();
        crate::store::add_translation_file("minecraft", &json, locale);
    }

    if !downloaded.bedrock.is_empty() {
        // Bedrock .lang files are already parsed into a HashMap at download time.
        // We load them entry by entry since they may use different key formats.
        for (key, value) in &downloaded.bedrock {
            crate::store::add_translation("bedrock_minecraft", key, value.as_str(), locale);
        }
    }

    if downloaded.has_any() {
        info!(
            "Loaded downloaded translations for {} (pumpkin: {}, java: {}, bedrock: {})",
            locale.to_code(),
            downloaded.pumpkin.len(),
            downloaded.java.len(),
            downloaded.bedrock.len(),
        );
    }
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

/// Fetch and parse a Bedrock `.lang` file from the given URL.
///
/// Bedrock lang files use `key=value` format, one entry per line.
/// Keys are lowercased for case-insensitive lookup.
///
/// After downloading, the data is verified against a `.sha256` checksum file
/// if one is available on the mirror — unless `skip_checksum` is `true`.
fn fetch_bedrock_lang(
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

    let mut map = HashMap::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            map.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    if map.is_empty() {
        return Err("no entries found in .lang file".to_string());
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_config_defaults() {
        let config = DownloadConfig::default();
        assert!(config.mirror_url.is_empty());
        assert_eq!(config.timeout_ms, 1000);
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
