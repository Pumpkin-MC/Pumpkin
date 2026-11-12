//! # Pumpkin Plugin Utilities (`pumpkin-plugin-utils`)
//!
//! A fast, secure, and developer-friendly utility crate for Pumpkin server plugins, providing:
//! - **Automatic Metadata Caching**: Call `init(context)` once on load; marketplace metadata is retrieved from host WIT and cached globally.
//! - **Zero-Argument Crashlytics**: `init(context)` automatically installs a panic hook that captures uncaught panics (`file:line:col`, message, and active hook context), scrubs PII, and transmits signed crash reports to `/api/v1/rest/telemetry/crash`. Also provides `record_error` / `report_crash` for handled errors.
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

/// Crashlytics reporting, automatic panic hook capture, and offline crash queue management.
#[cfg(feature = "crashlytics")]
pub mod crashlytics;
/// HTTP client helpers for marketplace interaction.
#[cfg(feature = "http")]
pub mod http;
/// License checking, validation, and lease management.
pub mod license;
/// Data models for metadata, licenses, updates, and crashlytics.
pub mod models;
/// Non-blocking update checks against marketplace endpoints.
#[cfg(feature = "updater")]
pub mod updater;

#[cfg(feature = "crashlytics")]
pub use crashlytics::{
    CrashReporter, CrashlyticsError, get_crash_context, install_panic_hook, scrub_crash_pii,
    set_crash_context, sign_telemetry_payload, with_crash_context,
};
#[cfg(feature = "license")]
pub use license::LicenseChecker;
pub use license::LicenseError;
pub use models::{
    CheckLicenseResponse, CheckUpdateResponse, CrashBacktraceFrame, CrashReportPayload,
    CrashReportResponse, DEFAULT_MARKETPLACE_URL, LicenseLease, LicenseStatus, PumpkinMetadata,
    RuntimeEnvironment,
};
#[cfg(feature = "updater")]
pub use updater::{UpdateChecker, UpdateError};

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

/// Global cache for verified plugin metadata.
static GLOBAL_METADATA: OnceLock<PumpkinMetadata> = OnceLock::new();
/// Global cache for plugin data folder path.
static GLOBAL_DATA_FOLDER: OnceLock<PathBuf> = OnceLock::new();
/// Global cache for host runtime environment metadata.
static GLOBAL_RUNTIME_ENV: OnceLock<Mutex<RuntimeEnvironment>> = OnceLock::new();

fn runtime_env_slot() -> &'static Mutex<RuntimeEnvironment> {
    GLOBAL_RUNTIME_ENV.get_or_init(|| Mutex::new(RuntimeEnvironment::default()))
}

/// Updates the globally cached host runtime environment reported with Crashlytics payloads.
pub fn set_runtime_environment(env: RuntimeEnvironment) {
    if let Ok(mut guard) = runtime_env_slot().lock() {
        *guard = env;
    }
}

/// Returns the globally cached host runtime environment.
#[must_use]
pub fn get_runtime_environment() -> RuntimeEnvironment {
    runtime_env_slot()
        .lock()
        .map_or_else(|_| RuntimeEnvironment::default(), |g| g.clone())
}

/// Initializes `pumpkin-plugin-utils` using the plugin's runtime `Context`.
///
/// Retrieves verified marketplace metadata provided by the host if the plugin is signed,
/// caches host environment information, installs the automatic Crashlytics panic hook,
/// and flushes any pending offline crash report from a previous run.
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
        let sys_info = context.get_server().get_sys_info();
        let os_str = match (sys_info.os_name, sys_info.os_version) {
            (Some(name), Some(ver)) if !ver.is_empty() => Some(format!("{name} {ver}")),
            (Some(name), _) => Some(name),
            (None, Some(ver)) => Some(ver),
            (None, None) => Some("Linux".to_string()),
        };
        set_runtime_environment(RuntimeEnvironment {
            server_type: "pumpkin".to_string(),
            server_version: sys_info.pumpkin_version,
            minecraft_version: None,
            os: os_str,
            arch: Some("x86_64".to_string()),
        });

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
/// Automatically installs the Crashlytics panic hook and flushes any previously queued crash report.
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

    #[cfg(feature = "crashlytics")]
    {
        install_panic_hook();
        let _ = flush_pending_crashes();
    }

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
#[cfg(feature = "license")]
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
#[cfg(feature = "updater")]
pub fn check_for_updates() -> Result<CheckUpdateResponse, UpdateError> {
    let meta = metadata().map_err(|_| UpdateError::NotInitialized)?;
    UpdateChecker::new().check_for_updates(&meta.plugin_name, &meta.version, &meta.marketplace_url)
}

/// Evaluates the complete offline license status (metadata + lease cache + grace period)
/// using the globally cached plugin data.
#[cfg(feature = "license")]
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

/// Reports a crash or non-fatal trap to Pumpkin Marketplace Crashlytics using the globally cached
/// plugin metadata (`POST /api/v1/rest/telemetry/crash`).
///
/// # Errors
///
/// Returns `CrashlyticsError` if `init(context)` was not called, the plugin lacks a marketplace
/// signature proof, or the HTTP request fails.
#[cfg(feature = "crashlytics")]
pub fn report_crash(
    trap_code: &str,
    error_message: &str,
    location: Option<&str>,
    entry_hook: Option<&str>,
    backtrace: Option<Vec<CrashBacktraceFrame>>,
) -> Result<CrashReportResponse, CrashlyticsError> {
    let meta = metadata().map_err(|_| CrashlyticsError::NotInitialized)?;
    let folder = get_data_folder().ok_or(CrashlyticsError::NotInitialized)?;
    let env = get_runtime_environment();
    let reporter = CrashReporter::new(folder);
    reporter.report_crash(
        meta,
        &env,
        trap_code,
        error_message,
        location,
        entry_hook,
        backtrace,
    )
}

/// Convenience helper (similar to Firebase Crashlytics `recordException`) for reporting a
/// handled or non-fatal plugin error using the caller's source location.
///
/// # Errors
///
/// Returns `CrashlyticsError` if `init(context)` was not called or transmission fails.
#[cfg(feature = "crashlytics")]
#[track_caller]
pub fn record_error(
    error_message: impl AsRef<str>,
) -> Result<CrashReportResponse, CrashlyticsError> {
    let caller = std::panic::Location::caller();
    let loc = format!("{}:{}:{}", caller.file(), caller.line(), caller.column());
    report_crash("panic", error_message.as_ref(), Some(&loc), None, None)
}

/// Convenience helper for reporting a handled or non-fatal plugin error associated with a
/// specific event hook or command (e.g. `"on-player-join"` or `"/pay"`), automatically
/// capturing the caller's source file, line, and column.
///
/// # Errors
///
/// Returns `CrashlyticsError` if `init(context)` was not called or transmission fails.
#[cfg(feature = "crashlytics")]
#[track_caller]
pub fn record_error_at(
    entry_hook: &str,
    error_message: impl AsRef<str>,
) -> Result<CrashReportResponse, CrashlyticsError> {
    let caller = std::panic::Location::caller();
    let loc = format!("{}:{}:{}", caller.file(), caller.line(), caller.column());
    report_crash(
        "panic",
        error_message.as_ref(),
        Some(&loc),
        Some(entry_hook),
        None,
    )
}

/// Flushes any previously queued offline crash report (`pending_crash.json`) to Pumpkin Marketplace.
///
/// # Errors
///
/// Returns `CrashlyticsError` if `init(context)` was not called or flushing fails.
#[cfg(feature = "crashlytics")]
pub fn flush_pending_crashes() -> Result<Option<CrashReportResponse>, CrashlyticsError> {
    let meta = metadata().map_err(|_| CrashlyticsError::NotInitialized)?;
    let folder = get_data_folder().ok_or(CrashlyticsError::NotInitialized)?;
    let reporter = CrashReporter::new(folder);
    reporter.flush_pending_crash(meta)
}
