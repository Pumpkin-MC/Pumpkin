//! # Pumpkin Plugin Utilities (`pumpkin-plugin-utils`)
//!
//! A fast, secure, and developer-friendly utility crate for Pumpkin server plugins, providing:
//! - **Offline Ed25519 Signature Verification**: Validates plugin integrity and marketplace metadata on startup in `< 1ms`.
//! - **Automatic Metadata Caching**: Call `init(context)` once on load; metadata is verified and cached globally for all subsequent operations.
//! - **Zero-Argument Updates & Online Licensing**: Check licenses and updates against official Pumpkin Marketplace endpoints without manual arguments.
//! - **Online License Checks**: Verify active licenses with `https://market.pumpkinmc.org/api/v1/rest/check-license`.
//! - **License Checks & Grace Periods**: Local lease management (`license_lease.json`) to prevent outages during marketplace downtime.
//! - **Dynamic Public Key Resolution**: Resolves keys via host WIT import, local cache, or HTTPS fallback without hardcoding keys.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use pumpkin_plugin_api::{Plugin, Context, register_plugin};
//! use pumpkin_plugin_utils::{init, check_license_online, check_for_updates};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn new() -> Self { MyPlugin }
//!
//!     fn on_load(&self, context: &Context) -> Result<(), String> {
//!         // 1. Initialize plugin-utils (verifies signature & caches metadata globally)
//!         let metadata = pumpkin_plugin_utils::init(context)
//!             .map_err(|e| format!("Plugin initialization failed: {e}"))?;
//!
//!         // 2. Check license online against marketplace
//!         let license_check = pumpkin_plugin_utils::check_license_online(None)
//!             .map_err(|e| format!("License check failed: {e}"))?;
//!
//!         if !license_check.valid {
//!             return Err(format!("Invalid license status: {}", license_check.status));
//!         }
//!
//!         // 3. Check for updates (zero arguments required)
//!         if let Ok(update) = pumpkin_plugin_utils::check_for_updates() {
//!             if update.update_available {
//!                 println!("A new version is available: {:?}", update.latest_version);
//!             }
//!         }
//!
//!         Ok(())
//!     }
//! }
//!
//! register_plugin!(MyPlugin);
//! ```

#![warn(missing_docs)]
#![allow(
    clippy::undocumented_unsafe_blocks,
    clippy::option_if_let_else,
    clippy::collection_is_never_read,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::panic
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Cryptographic signature verification.
pub mod crypto;
/// HTTP client helpers for marketplace interaction.
pub mod http;
/// License checking, validation, and lease management.
pub mod license;
/// Data models for metadata, signatures, licenses, and updates.
pub mod models;
/// Dynamic marketplace public key resolution.
pub mod resolver;
/// Non-blocking update checks against marketplace endpoints.
pub mod updater;
/// WASM binary inspection and custom section extraction.
pub mod wasm;

pub use crypto::verify_signature;
pub use license::{LicenseChecker, LicenseError};
pub use models::{
    CheckLicenseResponse, CheckUpdateResponse, DEFAULT_MARKETPLACE_URL, LicenseLease,
    LicenseStatus, MarketplacePublicKeyResponse, PumpkinMetadata, WasmSignatureEnvelope,
};
pub use resolver::PublicKeyResolver;
pub use updater::{UpdateChecker, UpdateError};
pub use wasm::{
    ExtractedSections, WasmError, extract_sections, find_self_wasm, strip_pumpkin_sections,
};

use std::{
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Global cache for verified plugin metadata.
static GLOBAL_METADATA: OnceLock<PumpkinMetadata> = OnceLock::new();
/// Global cache for plugin data folder path.
static GLOBAL_DATA_FOLDER: OnceLock<PathBuf> = OnceLock::new();

/// Initializes `pumpkin-plugin-utils` using the plugin's runtime `Context`.
///
/// Automatically locates the plugin WASM binary, verifies its cryptographic Ed25519 signature
/// against the marketplace public key, and caches the verified metadata globally.
///
/// # Errors
///
/// Returns `LicenseError` if signature verification, section extraction, or public key resolution fails.
pub fn init(
    context: &pumpkin_plugin_api::Context,
) -> Result<&'static PumpkinMetadata, LicenseError> {
    let data_folder = PathBuf::from(context.get_data_folder());
    init_with_folder(data_folder)
}

/// Initializes `pumpkin-plugin-utils` with a specific data folder path.
///
/// # Errors
///
/// Returns `LicenseError` if signature verification or section extraction fails.
pub fn init_with_folder(
    data_folder: impl AsRef<Path>,
) -> Result<&'static PumpkinMetadata, LicenseError> {
    let folder = data_folder.as_ref().to_path_buf();
    let checker = LicenseChecker::new(&folder);
    let metadata = checker.verify_self_offline()?;

    let _ = GLOBAL_DATA_FOLDER.set(folder);
    let _ = GLOBAL_METADATA.set(metadata);

    GLOBAL_METADATA.get().ok_or(LicenseError::NotInitialized)
}

/// Initializes `pumpkin-plugin-utils` with raw WASM bytes directly (useful for tests or embedded bytes).
///
/// # Errors
///
/// Returns `LicenseError` if signature verification fails.
pub fn init_with_bytes(
    wasm_bytes: &[u8],
    data_folder: impl AsRef<Path>,
) -> Result<&'static PumpkinMetadata, LicenseError> {
    let folder = data_folder.as_ref().to_path_buf();
    let checker = LicenseChecker::new(&folder);
    let metadata = checker.verify_offline(wasm_bytes)?;

    let _ = GLOBAL_DATA_FOLDER.set(folder);
    let _ = GLOBAL_METADATA.set(metadata);

    GLOBAL_METADATA.get().ok_or(LicenseError::NotInitialized)
}

/// Returns a reference to the globally cached, verified metadata if `init` has been called.
#[must_use]
pub fn get_metadata() -> Option<&'static PumpkinMetadata> {
    GLOBAL_METADATA.get()
}

/// Returns a reference to the globally cached metadata.
///
/// # Errors
///
/// Returns `LicenseError::NotInitialized` if `init(context)` has not been called yet.
pub fn metadata() -> Result<&'static PumpkinMetadata, LicenseError> {
    GLOBAL_METADATA.get().ok_or(LicenseError::NotInitialized)
}

/// Returns a reference to the globally cached plugin data folder if initialized.
#[must_use]
pub fn get_data_folder() -> Option<&'static Path> {
    GLOBAL_DATA_FOLDER.get().map(PathBuf::as_path)
}

/// Checks the license online against the marketplace REST API:
/// `GET /api/v1/rest/check-license?plugin_name={name}&license_key={key}`
///
/// If `license_key_override` is `None`, uses the `license_key` stored in the verified metadata.
///
/// # Errors
///
/// Returns `LicenseError` if querying the marketplace fails or if `init` was not called.
pub fn check_license_online(
    license_key_override: Option<&str>,
) -> Result<CheckLicenseResponse, LicenseError> {
    let meta = metadata()?;
    let folder = get_data_folder().ok_or(LicenseError::NotInitialized)?;
    let checker = LicenseChecker::new(folder);
    checker.check_license_online(meta, license_key_override)
}

/// Checks for updates against the marketplace using the globally cached plugin metadata:
/// `GET /api/v1/rest/check-update?plugin_name={name}&current_version={version}`
///
/// # Errors
///
/// Returns `UpdateError` if querying the marketplace fails or if `init` was not called.
pub fn check_for_updates() -> Result<CheckUpdateResponse, UpdateError> {
    let meta = metadata().map_err(|_| UpdateError::NotInitialized)?;
    UpdateChecker::new().check_for_updates(&meta.plugin_name, &meta.version, &meta.marketplace_url)
}

/// Evaluates the complete offline license status (offline check + lease cache + grace period)
/// using the globally cached plugin data.
#[must_use]
pub fn evaluate_license(grace_period_days: u32) -> LicenseStatus {
    let Some(folder) = get_data_folder() else {
        return LicenseStatus::Invalid("pumpkin_plugin_utils has not been initialized".to_string());
    };
    let checker = LicenseChecker::new(folder);
    match wasm::find_self_wasm(folder) {
        Ok(bytes) => checker.evaluate_license(&bytes, grace_period_days),
        Err(e) => LicenseStatus::Invalid(e.to_string()),
    }
}
