//! # Pumpkin Plugin Utilities (`pumpkin-plugin-utils`)
//!
//! A fast, secure, and developer-friendly utility crate for Pumpkin server plugins, providing:
//! - **Automatic Metadata Caching**: Call `init(context)` once on load; marketplace metadata is retrieved from host WIT and cached globally.
//! - **Zero-Argument Updates & Online Licensing**: Check licenses and updates against official Pumpkin Marketplace endpoints without manual arguments.
//! - **Online License Checks**: Verify active licenses with `https://market.pumpkinmc.org/api/v1/rest/check-license`.
//! - **License Checks & Grace Periods**: Local lease management (`license_lease.json`) to prevent outages during marketplace downtime.
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
//!         // 1. Initialize plugin-utils (retrieves verified marketplace metadata from host)
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

/// HTTP client helpers for marketplace interaction.
pub mod http;
/// License checking, validation, and lease management.
pub mod license;
/// Data models for metadata, licenses, and updates.
pub mod models;
/// Non-blocking update checks against marketplace endpoints.
pub mod updater;

pub use license::{LicenseChecker, LicenseError};
pub use models::{
    CheckLicenseResponse, CheckUpdateResponse, DEFAULT_MARKETPLACE_URL, LicenseLease,
    LicenseStatus, PumpkinMetadata,
};
pub use updater::{UpdateChecker, UpdateError};

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
/// Retrieves verified marketplace metadata provided by the host if the plugin is signed,
/// and caches it globally.
///
/// # Errors
///
/// Returns `LicenseError::UnsignedPlugin` if the plugin is not signed or marketplace metadata is missing.
pub fn init(
    context: &pumpkin_plugin_api::Context,
) -> Result<&'static PumpkinMetadata, LicenseError> {
    let data_folder = PathBuf::from(context.get_data_folder());

    #[cfg(target_arch = "wasm32")]
    {
        if let Some(market_meta) = context.get_marketplace_metadata() {
            let meta: PumpkinMetadata = market_meta.into();
            return init_with_metadata(meta, data_folder);
        }
        Err(LicenseError::UnsignedPlugin)
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = data_folder;
        GLOBAL_METADATA.get().ok_or(LicenseError::NotInitialized)
    }
}

/// Initializes `pumpkin-plugin-utils` with explicit metadata (useful for tests or custom initialization).
///
/// # Errors
///
/// Returns `LicenseError::NotInitialized` if caching fails.
pub fn init_with_metadata(
    metadata: PumpkinMetadata,
    data_folder: impl AsRef<Path>,
) -> Result<&'static PumpkinMetadata, LicenseError> {
    let folder = data_folder.as_ref().to_path_buf();
    let _ = GLOBAL_DATA_FOLDER.set(folder);
    let _ = GLOBAL_METADATA.set(metadata);

    GLOBAL_METADATA.get().ok_or(LicenseError::NotInitialized)
}

/// Returns a reference to the globally cached metadata if `init` has been called.
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

/// Evaluates the complete offline license status (metadata + lease cache + grace period)
/// using the globally cached plugin data.
#[must_use]
pub fn evaluate_license(grace_period_days: u32) -> LicenseStatus {
    let Some(folder) = get_data_folder() else {
        return LicenseStatus::Invalid("pumpkin_plugin_utils has not been initialized".to_string());
    };
    let Some(meta) = get_metadata() else {
        return LicenseStatus::Invalid("pumpkin_plugin_utils has not been initialized".to_string());
    };
    let checker = LicenseChecker::new(folder);
    checker.evaluate_license(meta, grace_period_days)
}
