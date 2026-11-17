//! Data models for Pumpkin plugin licensing, metadata, and marketplace endpoints.

use serde::{Deserialize, Serialize};

/// Default Pumpkin Marketplace URL.
pub const DEFAULT_MARKETPLACE_URL: &str = "https://market.pumpkinmc.org";

/// Default Pumpkin Telemetry & Crashlytics URL.
pub const DEFAULT_TELEMETRY_URL: &str = "https://telemetry.pumpkinmc.org";

/// Metadata embedded in a Pumpkin WASM plugin by the marketplace or developer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PumpkinMetadata {
    /// The marketplace base URL where this plugin is registered.
    pub marketplace_url: String,
    /// The unique plugin ID on the marketplace.
    pub plugin_id: i64,
    /// The canonical plugin name.
    pub plugin_name: String,
    /// The semver version string of the plugin.
    pub version: String,
    /// Developer ID.
    pub dev_id: i64,
    /// Developer display name or username.
    pub dev_name: String,
    /// Whether this is a paid marketplace plugin.
    pub is_paid: bool,
    /// The buyer / licensee user ID (0 for free/open-source).
    pub user_id: i64,
    /// Unique license key issued to the buyer, if paid.
    pub license_key: Option<String>,
    /// ISO-8601 timestamp of when this binary/license was issued.
    pub issued_at: String,
    /// Optional marketplace Ed25519 signature hex over the metadata for authenticated crash/telemetry reporting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_signature_hex: Option<String>,
}

impl From<pumpkin_plugin_api::MarketplaceMetadata> for PumpkinMetadata {
    fn from(m: pumpkin_plugin_api::MarketplaceMetadata) -> Self {
        let (clean_issued_at, extracted_sig) = match m.issued_at.split_once('#') {
            Some((ts, sig)) if !sig.trim().is_empty() => {
                (ts.to_string(), Some(sig.trim().to_string()))
            }
            _ => (m.issued_at, None),
        };
        Self {
            marketplace_url: m.marketplace_url,
            plugin_id: m.plugin_id,
            plugin_name: m.plugin_name,
            version: m.version,
            dev_id: m.dev_id,
            dev_name: m.dev_name,
            is_paid: m.is_paid,
            user_id: m.user_id,
            license_key: m.license_key,
            issued_at: clean_issued_at,
            wasm_signature_hex: extracted_sig,
        }
    }
}

/// Result of evaluating a plugin's license.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseStatus {
    /// The license is completely valid.
    Valid(PumpkinMetadata),
    /// Operating in an offline grace period with valid cached lease.
    GracePeriod {
        /// The metadata.
        metadata: PumpkinMetadata,
        /// Remaining days in the grace period.
        days_remaining: u32,
        /// Reason for operating in grace period (e.g. market unreachable).
        reason: String,
    },
    /// The license is invalid, expired, revoked, or tampered.
    Invalid(String),
    /// The plugin binary has no signature or metadata attached.
    Unsigned,
}

/// Cached license verification lease stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicenseLease {
    /// Plugin name.
    pub plugin_name: String,
    /// License key verified.
    pub license_key: Option<String>,
    /// Status string returned by the marketplace ("valid", "invalid", "revoked").
    pub status: String,
    /// Unix timestamp (seconds) when this lease was verified online.
    pub last_verified_timestamp: u64,
    /// Unix timestamp (seconds) until which this offline lease is valid.
    pub expires_timestamp: u64,
}

/// Response returned by the marketplace `/api/v1/rest/check-license` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckLicenseResponse {
    /// Whether the license is valid and active for this plugin.
    pub valid: bool,
    /// Human-readable status ("valid", "invalid", "revoked").
    pub status: String,
}

/// Response returned by the marketplace `/api/v1/rest/check-update` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckUpdateResponse {
    /// Whether a newer stable release exists on the marketplace.
    pub update_available: bool,
    /// The latest stable version string, if one exists.
    pub latest_version: Option<String>,
}

/// Host runtime environment metadata reported alongside crash reports.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeEnvironment {
    /// Host runtime type (`"pumpkin"` or `"vine"`).
    pub server_type: String,
    /// Host software version (e.g. `"0.2.2"`).
    pub server_version: String,
    /// Target Minecraft version if known (e.g. `"1.21.4"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minecraft_version: Option<String>,
    /// Host operating system name (e.g. `"Linux"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Host CPU architecture (e.g. `"x86_64"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
}

impl Default for RuntimeEnvironment {
    fn default() -> Self {
        Self {
            server_type: "pumpkin".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            minecraft_version: None,
            os: Some("Linux".to_string()),
            arch: Some("x86_64".to_string()),
        }
    }
}

/// A single WebAssembly stack trace frame attached to a crash report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashBacktraceFrame {
    /// Optional WebAssembly function index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub func_index: Option<u32>,
    /// Optional demangled function symbol name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub func_name: Option<String>,
    /// Optional WebAssembly module or component name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    /// Optional byte offset within the WASM code section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_offset: Option<u64>,
}

/// Payload transmitted to `POST /v1/crash` on Pumpkin Telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashReportPayload {
    /// Verified `pumpkin.metadata` extracted from the WASM binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PumpkinMetadata>,
    /// Marketplace Ed25519 signature hex over the metadata proving binary authenticity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_signature_hex: Option<String>,
    /// Host runtime environment (`"pumpkin"` or `"vine"`).
    pub server_type: String,
    /// Host software version.
    pub server_version: String,
    /// Target Minecraft version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minecraft_version: Option<String>,
    /// Host OS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// Host CPU architecture.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// Trap classification (`"panic"`, `"unreachable"`, `"memory_out_of_bounds"`, `"out_of_fuel"`, `"stack_overflow"`).
    pub trap_code: String,
    /// WIT export, event hook, or command that triggered the crash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_hook: Option<String>,
    /// Scrubbed guest panic message or trap string.
    pub error_message: String,
    /// Optional guest source file and line (`"src/lib.rs:42:9"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Optional structured WASM stack trace frames.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<Vec<CrashBacktraceFrame>>,
    /// Number of identical occurrences batched since the last report (`1..=25`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurrence_count: Option<u32>,
}

/// Response returned by `POST /v1/crash`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashReportResponse {
    /// Processing status (`"recorded"`, `"throttled"`, `"ignored_license"`, `"ignored_unlicensed"`).
    pub status: String,
    /// Human-readable status message.
    pub message: String,
    /// Computed 64-character SHA-256 deduplication fingerprint.
    #[serde(default)]
    pub fingerprint: Option<String>,
    /// Database ID of the deduplicated crash group.
    #[serde(default)]
    pub crash_group_id: Option<i64>,
}
