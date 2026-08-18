//! Data models for Pumpkin plugin licensing, metadata, and marketplace endpoints.

use serde::{Deserialize, Serialize};

/// Custom section names used in Pumpkin WASM plugin binaries.
pub const PUMPKIN_METADATA_SECTION: &str = "pumpkin.metadata";
/// Custom section name for the W3C/Pumpkin Ed25519 signature.
pub const WASM_SIGNATURE_SECTION: &str = "wasm_signature";
/// Legacy section name for signature backwards-compatibility.
pub const LEGACY_SIGNATURE_SECTION: &str = "pumpkin.signature";

/// Default Pumpkin Marketplace URL.
pub const DEFAULT_MARKETPLACE_URL: &str = "https://market.pumpkinmc.org";

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
}

/// Standard W3C Wasm-Sign signature envelope structure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WasmSignatureEnvelope {
    /// Signature envelope schema version.
    pub version: u8,
    /// Signature algorithm (e.g. "Ed25519").
    pub algorithm: String,
    /// Hex-encoded public key of the signer.
    pub public_key_hex: String,
    /// Hex-encoded signature bytes.
    pub signature_hex: String,
}

/// Result of evaluating a plugin's license.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LicenseStatus {
    /// The license is completely valid and verified.
    Valid(PumpkinMetadata),
    /// Operating in an offline grace period with valid cached lease.
    GracePeriod {
        /// The verified metadata.
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

/// Response returned by the marketplace `/api/v1/rest/public-key` endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketplacePublicKeyResponse {
    /// Cryptographic algorithm (e.g. "Ed25519").
    pub algorithm: String,
    /// Hex-encoded public key.
    pub public_key_hex: String,
}
