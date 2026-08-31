//! Data models for Pumpkin plugin licensing, metadata, and marketplace endpoints.

use serde::{Deserialize, Serialize};

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

impl From<pumpkin_plugin_api::MarketplaceMetadata> for PumpkinMetadata {
    fn from(m: pumpkin_plugin_api::MarketplaceMetadata) -> Self {
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
            issued_at: m.issued_at,
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
