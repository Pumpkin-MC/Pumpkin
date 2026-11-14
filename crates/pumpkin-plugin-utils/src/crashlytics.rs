//! Crashlytics reporting, automatic panic hook capture, and offline crash queue management.

use crate::{
    http::{HttpClient, HttpError},
    models::{
        CrashBacktraceFrame, CrashReportPayload, CrashReportResponse, DEFAULT_MARKETPLACE_URL,
        DEFAULT_TELEMETRY_URL, PumpkinMetadata, RuntimeEnvironment,
    },
};
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, Once, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tracing::{debug, warn};

/// Minimum cooldown in seconds between transmitting reports for the same plugin installation.
pub const CRASH_REPORT_COOLDOWN_SECS: u64 = 30;

/// Filename used to persist the plugin's Crashlytics Ed25519 identity key inside its data folder.
pub const CRASHLYTICS_IDENTITY_KEY_FILE: &str = "crashlytics_identity.key";

/// Filename used to persist an unsent crash report if the network is unreachable or the guest aborts.
pub const PENDING_CRASH_FILE: &str = "pending_crash.json";

/// Errors that can occur when building or transmitting a Crashlytics report.
#[derive(Debug, Error)]
pub enum CrashlyticsError {
    /// `pumpkin_plugin_utils::init(context)` has not been called yet.
    #[error(
        "Plugin-utils has not been initialized (call pumpkin_plugin_utils::init(context) first)"
    )]
    NotInitialized,
    /// The plugin metadata is missing the marketplace Ed25519 signature proof.
    #[error("Plugin metadata is missing the marketplace Ed25519 signature proof")]
    MissingSignatureProof,
    /// HTTP communication error with the marketplace crash ingestion endpoint.
    #[error("Failed to transmit crash report: {0}")]
    Http(#[from] HttpError),
    /// JSON serialization or deserialization error.
    #[error("Failed to serialize or parse crash report JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Local storage I/O error.
    #[error("I/O error accessing crashlytics storage: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
struct CrashThrottleState {
    last_sent_timestamp: u64,
    pending_counts: HashMap<String, u32>,
}

static THROTTLE_STATE: OnceLock<Mutex<CrashThrottleState>> = OnceLock::new();
static ACTIVE_ENTRY_HOOK: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static PANIC_HOOK_ONCE: Once = Once::new();

fn throttle_state() -> &'static Mutex<CrashThrottleState> {
    THROTTLE_STATE.get_or_init(|| Mutex::new(CrashThrottleState::default()))
}

fn active_entry_hook_slot() -> &'static Mutex<Option<String>> {
    ACTIVE_ENTRY_HOOK.get_or_init(|| Mutex::new(None))
}

/// Sets the active execution context hook (e.g. `"on-player-join"` or `"/eco pay"`) attached to
/// any subsequent panic or crash report.
pub fn set_crash_context(entry_hook: Option<&str>) {
    if let Ok(mut guard) = active_entry_hook_slot().lock() {
        *guard = entry_hook
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned);
    }
}

/// Returns the currently active execution context hook, if set.
#[must_use]
pub fn get_crash_context() -> Option<String> {
    active_entry_hook_slot()
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Executes a closure `f` with `entry_hook` set as the active Crashlytics execution context,
/// restoring the previous context when `f` completes.
pub fn with_crash_context<F, R>(entry_hook: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let prev = get_crash_context();
    set_crash_context(Some(entry_hook));
    let result = f();
    set_crash_context(prev.as_deref());
    result
}

fn looks_like_ipv4(token: &str) -> bool {
    let trimmed = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.');
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts
        .iter()
        .all(|p| !p.is_empty() && p.len() <= 3 && p.parse::<u8>().is_ok())
}

/// Scrubs sensitive tokens, license keys, IPv4 addresses, and email addresses from crash messages
/// before they leave the plugin sandbox.
#[must_use]
pub fn scrub_crash_pii(input: &str, max_len: usize) -> String {
    let clean_chars: String = input
        .chars()
        .filter(|c| (!c.is_control() || *c == ' ') && *c != '<' && *c != '>')
        .collect();

    let mut out_words = Vec::new();
    for word in clean_chars.split_whitespace() {
        let stripped = word.trim_matches(|c: char| {
            matches!(
                c,
                ',' | ';' | ':' | '(' | ')' | '[' | ']' | '{' | '}' | '"' | '\''
            )
        });
        let upper = stripped.to_ascii_uppercase();

        if upper.starts_with("PPPP-") && stripped.len() >= 19 {
            out_words.push("[REDACTED_LICENSE]".to_string());
        } else if stripped.starts_with("pmk_") && stripped.len() >= 12 {
            out_words.push("[REDACTED_API_KEY]".to_string());
        } else if stripped.starts_with("eyJ") && stripped.len() >= 24 && stripped.contains('.') {
            out_words.push("[REDACTED_TOKEN]".to_string());
        } else if looks_like_ipv4(stripped) {
            out_words.push("[REDACTED_IP]".to_string());
        } else if stripped.contains('@')
            && !stripped.starts_with("pumpkin:")
            && !stripped.starts_with("wasi:")
            && stripped.split_once('@').is_some_and(|(l, r)| {
                !l.is_empty()
                    && r.contains('.')
                    && !r.starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'])
            })
        {
            out_words.push("[REDACTED_EMAIL]".to_string());
        } else {
            out_words.push(word.to_string());
        }
    }

    let joined = out_words.join(" ");
    joined.chars().take(max_len).collect()
}

/// Computes the signed telemetry payload layout `{timestamp}.{body_bytes}` and signs it with
/// the given Ed25519 `SigningKey`, returning `(public_key_hex, signature_hex)`.
#[must_use]
pub fn sign_telemetry_payload(
    signing_key: &SigningKey,
    timestamp_str: &str,
    body_bytes: &[u8],
) -> (String, String) {
    let mut signed_data = Vec::with_capacity(timestamp_str.len() + 1 + body_bytes.len());
    signed_data.extend_from_slice(timestamp_str.as_bytes());
    signed_data.push(b'.');
    signed_data.extend_from_slice(body_bytes);

    let signature = signing_key.sign(&signed_data);
    let pubkey_hex = hex::encode(signing_key.verifying_key().to_bytes());
    let sig_hex = hex::encode(signature.to_bytes());
    (pubkey_hex, sig_hex)
}

/// Manages Crashlytics identity keys, local deduplication batching, offline crash persistence,
/// and signed crash report transmission to Pumpkin Marketplace.
pub struct CrashReporter {
    data_folder: PathBuf,
    http_client: HttpClient,
}

impl CrashReporter {
    /// Creates a new `CrashReporter` using the given plugin data folder.
    #[must_use]
    pub fn new(data_folder: impl AsRef<Path>) -> Self {
        Self {
            data_folder: data_folder.as_ref().to_path_buf(),
            http_client: HttpClient::default(),
        }
    }

    fn key_path(&self) -> PathBuf {
        self.data_folder.join(CRASHLYTICS_IDENTITY_KEY_FILE)
    }

    fn pending_crash_path(&self) -> PathBuf {
        self.data_folder.join(PENDING_CRASH_FILE)
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }

    /// Loads the persisted 32-byte Ed25519 identity key from `data/crashlytics_identity.key`,
    /// or derives and persists a deterministic installation key if the file does not exist yet.
    #[must_use]
    pub fn load_or_derive_signing_key(&self, metadata: &PumpkinMetadata) -> SigningKey {
        let path = self.key_path();
        if path.exists()
            && let Ok(bytes) = std::fs::read(&path)
            && bytes.len() == 32
        {
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&bytes);
            return SigningKey::from_bytes(&seed);
        }

        // Derive a stable installation seed from the plugin data folder + issued metadata
        // mixed with system time if creating a brand-new persisted key file.
        let mut hasher = Sha256::new();
        hasher.update(b"pumpkin_crashlytics_identity_v1:");
        hasher.update(self.data_folder.to_string_lossy().as_bytes());
        hasher.update(b":");
        hasher.update(metadata.plugin_id.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(metadata.user_id.to_string().as_bytes());
        hasher.update(b":");
        hasher.update(metadata.issued_at.as_bytes());

        let mut seed = [0u8; 32];
        seed.copy_from_slice(&hasher.finalize());

        if std::fs::create_dir_all(&self.data_folder).is_ok() {
            let _ = std::fs::write(&path, seed);
        }

        SigningKey::from_bytes(&seed)
    }

    /// Persists a crash payload to `pending_crash.json` in case the network request fails or
    /// the guest aborts immediately after a panic.
    pub fn write_pending_crash(&self, payload: &CrashReportPayload) {
        if std::fs::create_dir_all(&self.data_folder).is_ok()
            && let Ok(bytes) = serde_json::to_vec(payload)
        {
            let _ = std::fs::write(self.pending_crash_path(), bytes);
        }
    }

    /// Removes `pending_crash.json` after a crash report is successfully delivered.
    pub fn clear_pending_crash(&self) {
        let path = self.pending_crash_path();
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Flushes any previously queued `pending_crash.json` from disk to the marketplace.
    ///
    /// # Errors
    ///
    /// Returns `CrashlyticsError` if sending the queued report fails.
    pub fn flush_pending_crash(
        &self,
        metadata: &PumpkinMetadata,
    ) -> Result<Option<CrashReportResponse>, CrashlyticsError> {
        let path = self.pending_crash_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)?;
        let payload: CrashReportPayload = serde_json::from_slice(&bytes)?;
        let resp = self.send_payload(metadata, &payload)?;
        self.clear_pending_crash();
        Ok(Some(resp))
    }

    /// Resolves the crash ingestion URL, routing official Marketplace plugins to the isolated
    /// `https://telemetry.pumpkinmc.org/v1/crash` endpoint while preserving custom/local URLs.
    #[must_use]
    pub fn resolve_crash_endpoint(marketplace_url: &str) -> String {
        let base = marketplace_url.trim_end_matches('/');
        if base.is_empty() || base.eq_ignore_ascii_case(DEFAULT_MARKETPLACE_URL) {
            format!("{DEFAULT_TELEMETRY_URL}/v1/crash")
        } else {
            format!("{base}/v1/crash")
        }
    }

    /// Signs and transmits a `CrashReportPayload` to the telemetry crash endpoint (`POST /v1/crash`).
    ///
    /// # Errors
    ///
    /// Returns `CrashlyticsError` if the metadata has no marketplace signature or the HTTP call fails.
    pub fn send_payload(
        &self,
        metadata: &PumpkinMetadata,
        payload: &CrashReportPayload,
    ) -> Result<CrashReportResponse, CrashlyticsError> {
        let has_sig = payload
            .wasm_signature_hex
            .as_deref()
            .or(metadata.wasm_signature_hex.as_deref())
            .is_some_and(|s| !s.trim().is_empty())
            || metadata.issued_at.contains('#');

        if !has_sig {
            return Err(CrashlyticsError::MissingSignatureProof);
        }

        let signing_key = self.load_or_derive_signing_key(metadata);
        let json_bytes = serde_json::to_vec(payload)?;
        let timestamp_str = Self::current_timestamp().to_string();
        let (pubkey_hex, sig_hex) =
            sign_telemetry_payload(&signing_key, &timestamp_str, &json_bytes);

        let url = Self::resolve_crash_endpoint(&metadata.marketplace_url);
        debug!(
            "Transmitting plugin crash report for '{}' v{} to {}",
            metadata.plugin_name, metadata.version, url
        );

        let response_str = self.http_client.post_signed_telemetry(
            &url,
            &json_bytes,
            &pubkey_hex,
            &sig_hex,
            &timestamp_str,
        )?;
        let parsed: CrashReportResponse = serde_json::from_str(&response_str)?;
        Ok(parsed)
    }

    /// Builds, deduplicates/throttles, persists, and transmits a crash report.
    ///
    /// # Errors
    ///
    /// Returns `CrashlyticsError` if the report cannot be transmitted.
    #[allow(clippy::too_many_arguments)]
    pub fn report_crash(
        &self,
        metadata: &PumpkinMetadata,
        env: &RuntimeEnvironment,
        trap_code: &str,
        error_message: &str,
        location: Option<&str>,
        entry_hook: Option<&str>,
        backtrace: Option<Vec<CrashBacktraceFrame>>,
    ) -> Result<CrashReportResponse, CrashlyticsError> {
        let scrubbed_msg = scrub_crash_pii(error_message, 512);
        let scrubbed_loc = location
            .map(|l| scrub_crash_pii(l, 160))
            .filter(|l| !l.is_empty());
        let resolved_hook = entry_hook
            .map(ToOwned::to_owned)
            .or_else(get_crash_context)
            .map(|h| scrub_crash_pii(&h, 128))
            .filter(|h| !h.is_empty());

        let sig_hex = metadata.wasm_signature_hex.clone().or_else(|| {
            metadata
                .issued_at
                .split_once('#')
                .map(|(_, s)| s.trim().to_string())
                .filter(|s| !s.is_empty())
        });

        let dedup_key = format!(
            "{}:{}:{}:{}",
            trap_code,
            resolved_hook.as_deref().unwrap_or(""),
            scrubbed_loc.as_deref().unwrap_or(""),
            scrubbed_msg
        );

        let now = Self::current_timestamp();
        let mut batched_count = 1u32;

        if let Ok(mut guard) = throttle_state().lock() {
            if now.saturating_sub(guard.last_sent_timestamp) < CRASH_REPORT_COOLDOWN_SECS {
                let entry = guard.pending_counts.entry(dedup_key).or_insert(0);
                *entry = entry.saturating_add(1).min(25);
                return Ok(CrashReportResponse {
                    status: "throttled_local".to_string(),
                    message: "Crash batched locally due to 30s per-server cooldown".to_string(),
                    fingerprint: None,
                    crash_group_id: None,
                });
            }
            if let Some(prev_count) = guard.pending_counts.remove(&dedup_key) {
                batched_count = batched_count.saturating_add(prev_count).clamp(1, 25);
            }
            guard.last_sent_timestamp = now;
        }

        let payload = CrashReportPayload {
            metadata: Some(metadata.clone()),
            wasm_signature_hex: sig_hex,
            server_type: env.server_type.clone(),
            server_version: env.server_version.clone(),
            minecraft_version: env.minecraft_version.clone(),
            os: env.os.clone(),
            arch: env.arch.clone(),
            trap_code: trap_code.to_string(),
            entry_hook: resolved_hook,
            error_message: if scrubbed_msg.is_empty() {
                format!("Wasmtime guest trap: {trap_code}")
            } else {
                scrubbed_msg
            },
            location: scrubbed_loc,
            backtrace,
            occurrence_count: Some(batched_count),
        };

        // Persist before attempting network transmission in case the guest terminates
        self.write_pending_crash(&payload);

        match self.send_payload(metadata, &payload) {
            Ok(resp) => {
                self.clear_pending_crash();
                Ok(resp)
            }
            Err(err) => {
                warn!("Crash report queued to disk after transmission error: {err}");
                Err(err)
            }
        }
    }
}

/// Installs the global Rust panic hook that automatically captures uncaught plugin panics
/// (message, source file/line/column, and active `entry_hook`) and reports them to
/// Pumpkin Marketplace Crashlytics.
///
/// Safe to call multiple times; the hook is registered at most once per plugin instance.
pub fn install_panic_hook() {
    PANIC_HOOK_ONCE.call_once(|| {
        let prev_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let msg = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Uncaught plugin guest panic".to_string()
            };

            let location = panic_info
                .location()
                .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()));

            if let (Some(meta), Some(folder)) = (crate::get_metadata(), crate::get_data_folder()) {
                let env = crate::get_runtime_environment();
                let reporter = CrashReporter::new(folder);
                let _ = reporter.report_crash(
                    meta,
                    &env,
                    "panic",
                    &msg,
                    location.as_deref(),
                    None,
                    None,
                );
            }

            prev_hook(panic_info);
        }));
    });
}
