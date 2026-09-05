//! License validation, leasing, and offline grace periods.

use crate::{
    http::{HttpClient, HttpError},
    models::{CheckLicenseResponse, LicenseLease, LicenseStatus, PumpkinMetadata},
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use thiserror::Error;
use tracing::{debug, info};

/// License checking and verification errors.
#[derive(Debug, Error)]
pub enum LicenseError {
    /// HTTP communication error with marketplace.
    #[error("Marketplace HTTP error: {0}")]
    Http(#[from] HttpError),
    /// Metadata validation error (e.g. missing license on paid plugin).
    #[error("License metadata mismatch: {0}")]
    MetadataMismatch(String),
    /// License revoked or refunded by marketplace.
    #[error("License was revoked or refunded: {0}")]
    Revoked(String),
    /// License is expired.
    #[error("License has expired on {0}")]
    Expired(String),
    /// I/O error reading/writing license cache.
    #[error("I/O error with license storage: {0}")]
    Io(#[from] std::io::Error),
    /// JSON serialization error.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    /// Plugin is unsigned or missing marketplace metadata.
    #[error("Plugin is unsigned or missing marketplace metadata")]
    UnsignedPlugin,
    /// Plugin has not been initialized.
    #[error(
        "Plugin-utils has not been initialized (call pumpkin_plugin_utils::init(context) first)"
    )]
    NotInitialized,
}

/// Manages license checks, cached leases, and offline grace periods.
pub struct LicenseChecker {
    data_folder: PathBuf,
    http_client: HttpClient,
}

impl LicenseChecker {
    /// Creates a new `LicenseChecker` instance for the given data folder.
    #[must_use]
    pub fn new(data_folder: impl AsRef<Path>) -> Self {
        let folder = data_folder.as_ref().to_path_buf();
        Self {
            data_folder: folder,
            http_client: HttpClient::default(),
        }
    }

    /// Path to the cached `license_lease.json` file.
    fn lease_path(&self) -> PathBuf {
        self.data_folder.join("license_lease.json")
    }

    /// Reads the cached license lease from disk.
    #[must_use]
    pub fn read_cached_lease(&self) -> Option<LicenseLease> {
        let path = self.lease_path();
        if path.exists() {
            if let Ok(data) = std::fs::read(path) {
                if let Ok(lease) = serde_json::from_slice::<LicenseLease>(&data) {
                    return Some(lease);
                }
            }
        }
        None
    }

    /// Saves a verified license lease to disk.
    ///
    /// # Errors
    ///
    /// Returns `LicenseError` if writing to disk fails.
    pub fn write_cached_lease(&self, lease: &LicenseLease) -> Result<(), LicenseError> {
        std::fs::create_dir_all(&self.data_folder)?;
        let json = serde_json::to_vec_pretty(lease)?;
        std::fs::write(self.lease_path(), json)?;
        Ok(())
    }

    /// Returns the current Unix timestamp in seconds.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }

    /// Evaluates the complete license status (metadata + lease cache + grace period).
    #[must_use]
    pub fn evaluate_license(
        &self,
        metadata: &PumpkinMetadata,
        grace_period_days: u32,
    ) -> LicenseStatus {
        // Free/Open-Source plugins are always valid
        if !metadata.is_paid {
            return LicenseStatus::Valid(metadata.clone());
        }

        if metadata.license_key.is_none() {
            return LicenseStatus::Invalid(
                "Paid plugin metadata is missing a license_key".to_string(),
            );
        }

        // For paid plugins, inspect cached lease
        let now = Self::current_timestamp();
        if let Some(lease) = self.read_cached_lease() {
            if lease
                .plugin_name
                .eq_ignore_ascii_case(&metadata.plugin_name)
                && lease.status == "valid"
                && now <= lease.expires_timestamp
            {
                return LicenseStatus::Valid(metadata.clone());
            }

            // Check if within grace period
            let grace_seconds = u64::from(grace_period_days) * 86400;
            if now <= lease.last_verified_timestamp + grace_seconds {
                let seconds_left =
                    (lease.last_verified_timestamp + grace_seconds).saturating_sub(now);
                let days_remaining = (seconds_left / 86400).max(1) as u32;
                return LicenseStatus::GracePeriod {
                    metadata: metadata.clone(),
                    days_remaining,
                    reason: "Operating in offline grace period with previous valid lease"
                        .to_string(),
                };
            }
        }

        // If no cached lease exists yet (first run) and offline, allow initial valid state
        LicenseStatus::Valid(metadata.clone())
    }

    /// Checks the license online against the marketplace REST API:
    /// `GET /api/v1/rest/check-license?plugin_name={name}&license_key={key}`
    ///
    /// # Errors
    ///
    /// Returns `LicenseError` if the HTTP request fails or no license key is available.
    pub fn check_license_online(
        &self,
        metadata: &PumpkinMetadata,
        license_key_override: Option<&str>,
    ) -> Result<CheckLicenseResponse, LicenseError> {
        let license_key = license_key_override
            .or(metadata.license_key.as_deref())
            .ok_or_else(|| {
                LicenseError::MetadataMismatch(
                    "No license key available in metadata or argument".to_string(),
                )
            })?;

        let url = format!(
            "{}/api/v1/rest/check-license?plugin_name={}&license_key={}",
            metadata.marketplace_url.trim_end_matches('/'),
            urlencoding(&metadata.plugin_name),
            urlencoding(license_key),
        );

        debug!("Checking license online at {url}");

        let response_str = self.http_client.get(&url)?;
        let check_response: CheckLicenseResponse = serde_json::from_str(&response_str)?;

        let now = Self::current_timestamp();
        let ttl_seconds = 86400 * 7; // 7 days lease cache

        let lease = LicenseLease {
            plugin_name: metadata.plugin_name.clone(),
            license_key: Some(license_key.to_string()),
            status: check_response.status.clone(),
            last_verified_timestamp: now,
            expires_timestamp: if check_response.valid {
                now + ttl_seconds
            } else {
                now
            },
        };

        let _ = self.write_cached_lease(&lease);

        if check_response.valid {
            info!(
                "Online license check successful for '{}' (status: {})",
                metadata.plugin_name, check_response.status
            );
        } else {
            info!(
                "Online license check returned invalid for '{}' (status: {})",
                metadata.plugin_name, check_response.status
            );
        }

        Ok(check_response)
    }
}

/// Minimal URL encoding helper for query parameters.
fn urlencoding(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}
