use std::{any::Any, sync::Arc};
use thiserror::Error;

use crate::{concurrent_store::LegacyStore, scheduler::TaskScheduler};

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Could not create the Wasmtime engine")]
    EngineCreationFailed(#[source] wasmtime::Error),
    #[error("Could not set up the plugin linker")]
    LinkerSetupFailed(#[source] wasmtime::Error),
    #[error("The plugin has no versioned pumpkin:plugin/metadata export")]
    UnknownApiVersion(),
    #[error("The plugin targets API {0}; this server supports {1}")]
    UnsupportedApiVersion(String, &'static str),
    #[error("Plugin API 0.1 supports WASI 0.2 only, but the plugin imports `{0}`")]
    UnsupportedWasi(String),
    #[error("The plugin does not match the expected Pumpkin API ({0})")]
    ApiMismatch(PluginApiVersion, #[source] wasmtime::Error),
    #[error("Could not read the plugin file")]
    FileReadFailed(#[source] std::io::Error),
    #[error("Could not compile the plugin component")]
    ComponentNewFailed(#[source] wasmtime::Error),
    #[error("Could not serialize the compiled plugin cache")]
    ComponentCacheSerializeFailed(#[source] wasmtime::Error),
    #[error("Could not write the compiled plugin cache")]
    ComponentCacheWriteFailed(#[source] std::io::Error),
    #[error("Could not instantiate the plugin")]
    InstantiationFailed(#[source] wasmtime::Error),
    #[error("The plugin failed during init_plugin")]
    CallInitPluginFailed(#[source] wasmtime::Error),
    #[error("The plugin failed while providing its metadata")]
    CallGetMetadataFailed(#[source] wasmtime::Error),
    #[error("Could not resolve the plugin path")]
    PathResolutionFailed(#[source] std::io::Error),
    #[error("Could not create the Wasmtime cache")]
    CacheCreationFailed(#[source] wasmtime::Error),
}

pub const SUPPORTED_PLUGIN_APIS: [(&str, PluginApiVersion); 2] = [
    ("0.1", PluginApiVersion::V0_1),
    ("0.2", PluginApiVersion::V0_2),
];

impl PluginInitError {
    /// Returns a hint when the error identifies a specific action.
    #[must_use]
    pub fn help(&self) -> Option<&'static str> {
        match self {
            Self::UnsupportedApiVersion(..) => {
                Some("Install a plugin built for one of the supported Pumpkin API versions.")
            }
            Self::ApiMismatch(..) => {
                Some("Install a plugin release built for this Pumpkin version.")
            }
            Self::FileReadFailed(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                Some("Check that the server can read the plugin file.")
            }
            Self::ComponentCacheWriteFailed(error)
                if error.kind() == std::io::ErrorKind::PermissionDenied =>
            {
                Some("Check that the server can write to the plugin cache directory.")
            }
            _ => None,
        }
    }

    /// Returns the original Wasmtime error, if this variant contains one.
    #[must_use]
    pub const fn wasmtime_error(&self) -> Option<&wasmtime::Error> {
        match self {
            Self::EngineCreationFailed(error)
            | Self::LinkerSetupFailed(error)
            | Self::ApiMismatch(_, error)
            | Self::ComponentNewFailed(error)
            | Self::ComponentCacheSerializeFailed(error)
            | Self::InstantiationFailed(error)
            | Self::CallInitPluginFailed(error)
            | Self::CallGetMetadataFailed(error)
            | Self::CacheCreationFailed(error) => Some(error),
            _ => None,
        }
    }
}

/// What a plugin component declares at its top level, read without compiling it.
///
/// A compiled `Component` belongs to one `Engine`, and the engine depends on the plugin's API
/// version, so this has to be known first.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct ComponentNames {
    /// The version of the exported `pumpkin:plugin/metadata` interface, such as `0.1.0`.
    pub metadata_version: Option<String>,
    /// The first imported WASI 0.3 interface, if any.
    pub wasi_p3_import: Option<String>,
}

/// Scans the top-level imports and exports of a component binary.
///
/// # Errors
/// Returns [`PluginInitError::ComponentNewFailed`] when the bytes are not a valid component.
pub fn scan_component_names(wasm_bytes: &[u8]) -> Result<ComponentNames, PluginInitError> {
    let malformed = |error: wasmparser::BinaryReaderError| {
        PluginInitError::ComponentNewFailed(wasmtime::Error::new(error))
    };
    let mut names = ComponentNames::default();
    // Nested components and modules repeat the same section kinds; only depth 0 is the plugin.
    let mut depth = 0usize;
    for payload in wasmparser::Parser::new(0).parse_all(wasm_bytes) {
        match payload.map_err(malformed)? {
            wasmparser::Payload::ModuleSection { .. }
            | wasmparser::Payload::ComponentSection { .. } => depth += 1,
            wasmparser::Payload::End(_) => depth = depth.saturating_sub(1),
            wasmparser::Payload::ComponentImportSection(imports) if depth == 0 => {
                for import in imports {
                    let name = import.map_err(malformed)?.name.full_name();
                    if names.wasi_p3_import.is_none() && is_wasi_p3_import(&name) {
                        names.wasi_p3_import = Some(name.into_owned());
                    }
                }
            }
            wasmparser::Payload::ComponentExportSection(exports) if depth == 0 => {
                for export in exports {
                    let name = export.map_err(malformed)?.name.full_name();
                    if let Some(version) = name.strip_prefix("pumpkin:plugin/metadata@") {
                        names.metadata_version = Some(version.to_owned());
                    }
                }
            }
            _ => {}
        }
    }
    Ok(names)
}

fn is_wasi_p3_import(name: &str) -> bool {
    name.starts_with("wasi:")
        && name
            .rsplit_once('@')
            .is_some_and(|(_, version)| version.starts_with("0.3."))
}

#[derive(Copy, Clone, Debug)]
pub enum PluginApiVersion {
    V0_1,
    V0_2,
}

impl std::fmt::Display for PluginApiVersion {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::V0_1 => "0.1.0",
            Self::V0_2 => "0.2.0",
        })
    }
}

pub struct WasmPlugin {
    pub name: String,
    pub version: String,
    /// The guest exports of the plugin, as generated for `api_version`.
    pub plugin_instance: Arc<dyn Any + Send + Sync>,
    pub api_version: PluginApiVersion,
    pub store: LegacyStore,
    pub task_scheduler: Arc<TaskScheduler>,
}

impl WasmPlugin {
    /// Logs the skipped callback and returns `true` if the plugin store is stopped.
    #[must_use]
    pub fn report_stopped(&self, operation: &str) -> bool {
        if !self.store.state().is_terminal() {
            return false;
        }
        tracing::error!(
            "Plugin \"{}\" ({}) is stopped; skipped {}.",
            pumpkin_core::plugin::error::inline(&self.name),
            pumpkin_core::plugin::error::inline(&self.version),
            pumpkin_core::plugin::error::inline(operation)
        );
        true
    }

    /// Logs a callback failure at ERROR and its original Wasmtime details at DEBUG.
    pub fn report_failure(&self, operation: &str, failure: &wasmtime::Error) {
        use pumpkin_core::plugin::error;
        let headline = format!(
            "Plugin \"{}\" ({}) failed during {}.",
            error::inline(&self.name),
            error::inline(&self.version),
            error::inline(operation)
        );
        let stopped = self.store.state().is_terminal();
        let help = crate::error::help(failure)
            .or_else(|| stopped.then_some("Reload the plugin or restart the server to retry."));
        let text = crate::error::message(
            &headline,
            &crate::error::causes(failure),
            stopped.then_some("The plugin has stopped."),
            help,
            Some(failure),
        );
        tracing::error!("{text}");
        error::debug_details(&headline, pumpkin_plugin_runtime::guest_call_error(failure));
    }

    /// The guest exports of this plugin. Each API version's host only ever sees plugins of its own version.
    ///
    /// # Panics
    /// Panics when `T` is not the type generated for this plugin's API version.
    #[must_use]
    pub fn instance<T: 'static>(&self) -> &T {
        self.plugin_instance
            .downcast_ref()
            .unwrap_or_else(|| panic!("Unexpected plugin version in host path."))
    }
}

impl Drop for WasmPlugin {
    fn drop(&mut self) {
        self.store.discard();
    }
}

#[cfg(test)]
mod tests {
    use super::{PluginApiVersion, PluginInitError, is_wasi_p3_import, scan_component_names};
    use std::io::{Error, ErrorKind};
    use wasm_encoder::{ComponentBuilder, ComponentTypeRef, InstanceType};

    #[test]
    fn help_is_limited_to_actionable_typed_failures() {
        assert!(
            PluginInitError::UnsupportedApiVersion("0.3.0".into(), "0.1 and 0.2")
                .help()
                .is_some()
        );
        assert!(
            PluginInitError::ApiMismatch(
                PluginApiVersion::V0_1,
                wasmtime::Error::msg("type mismatch")
            )
            .help()
            .is_some()
        );
        for kind in [ErrorKind::PermissionDenied, ErrorKind::NotFound] {
            let expected = kind == ErrorKind::PermissionDenied;
            assert_eq!(
                PluginInitError::FileReadFailed(Error::from(kind))
                    .help()
                    .is_some(),
                expected
            );
            assert_eq!(
                PluginInitError::ComponentCacheWriteFailed(Error::from(kind))
                    .help()
                    .is_some(),
                expected
            );
        }
        for error in [
            PluginInitError::EngineCreationFailed(wasmtime::Error::msg("failed")),
            PluginInitError::LinkerSetupFailed(wasmtime::Error::msg("failed")),
            PluginInitError::UnknownApiVersion(),
            PluginInitError::ComponentNewFailed(wasmtime::Error::msg("Permission denied")),
            PluginInitError::ComponentCacheSerializeFailed(wasmtime::Error::msg("failed")),
            PluginInitError::InstantiationFailed(wasmtime::Error::msg("failed")),
            PluginInitError::CallInitPluginFailed(wasmtime::Error::msg("failed")),
            PluginInitError::CallGetMetadataFailed(wasmtime::Error::msg("failed")),
            PluginInitError::PathResolutionFailed(Error::from(ErrorKind::PermissionDenied)),
            PluginInitError::CacheCreationFailed(wasmtime::Error::msg("failed")),
        ] {
            assert!(error.help().is_none(), "{error}");
        }
    }
    fn component_importing(names: &[&str]) -> Vec<u8> {
        let mut component = ComponentBuilder::default();
        for name in names {
            let ty = component.type_instance(None, &InstanceType::new());
            component.import(*name, ComponentTypeRef::Instance(ty));
        }
        component.finish()
    }

    #[test]
    fn wasi_p3_imports_are_recognised_by_version() {
        assert!(is_wasi_p3_import("wasi:random/random@0.3.0"));
        assert!(is_wasi_p3_import("wasi:cli/stdout@0.3.0-rc-2026-03-15"));
        assert!(!is_wasi_p3_import("wasi:random/random@0.2.6"));
        assert!(!is_wasi_p3_import("wasi:random/random"));
        assert!(!is_wasi_p3_import("pumpkin:plugin/context@0.3.0"));
    }

    #[test]
    fn scan_reports_the_first_wasi_p3_import() {
        let bytes = component_importing(&["wasi:random/random@0.2.6", "wasi:random/random@0.3.0"]);
        let names = scan_component_names(&bytes).expect("scan component");
        assert_eq!(
            names.wasi_p3_import.as_deref(),
            Some("wasi:random/random@0.3.0")
        );
        assert_eq!(names.metadata_version, None);
    }

    #[test]
    fn scan_rejects_bytes_that_are_not_a_component() {
        assert!(scan_component_names(b"not wasm").is_err());
    }
}
