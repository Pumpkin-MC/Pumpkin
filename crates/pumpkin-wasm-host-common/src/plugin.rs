use std::{any::Any, sync::Arc};
use thiserror::Error;

use crate::{concurrent_store::LegacyStore, scheduler::TaskScheduler};

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Engine creation failed: {0}")]
    EngineCreationFailed(wasmtime::Error),
    #[error("Failed to setup linker: {0}")]
    LinkerSetupFailed(wasmtime::Error),
    #[error("Could not identify the plugin API version (does not export pumpkin:plugin/metadata?)")]
    UnknownApiVersion(),
    #[error("Plugin is built against an unsupported version of the API: {0}")]
    UnsupportedApiVersion(String),
    #[error("Plugin API 0.1 supports WASI 0.2 only, but the plugin imports `{0}`")]
    UnsupportedWasi(String),
    #[error("Plugin was built against a different iteration of the API: {0}")]
    ApiMismatch(wasmtime::Error),
    #[error("Failed to read plugin file: {0}")]
    FileReadFailed(std::io::Error),
    #[error("Failed to load plugin as component: {0}")]
    ComponentNewFailed(wasmtime::Error),
    #[error("Failed to create cache data for plugin: {0}")]
    ComponentCacheSerializeFailed(wasmtime::Error),
    #[error("Failed to write cache file for plugin: {0}")]
    ComponentCacheWriteFailed(std::io::Error),
    #[error("Failed to instantiate plugin: {0}")]
    InstantiationFailed(wasmtime::Error),
    #[error("Calling `init_plugin` failed: {0}")]
    CallInitPluginFailed(wasmtime::Error),
    #[error("Calling `get_metadata` failed: {0}")]
    CallGetMetadataFailed(wasmtime::Error),
    #[error("Failed to get absolute path: {0}")]
    PathResolutionFailed(std::io::Error),
    #[error("Failed to create cache: {0}")]
    CacheCreationFailed(wasmtime::Error),
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
        PluginInitError::ComponentNewFailed(wasmtime::Error::msg(error.to_string()))
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

#[derive(Copy, Clone)]
pub enum PluginApiVersion {
    V0_1,
    V0_2,
}

pub struct WasmPlugin {
    /// The guest exports of the plugin, as generated for `api_version`.
    pub plugin_instance: Arc<dyn Any + Send + Sync>,
    pub api_version: PluginApiVersion,
    pub store: LegacyStore,
    pub task_scheduler: Arc<TaskScheduler>,
}

impl WasmPlugin {
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
    use wasm_encoder::{ComponentBuilder, ComponentTypeRef, InstanceType};

    use super::{is_wasi_p3_import, scan_component_names};

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
