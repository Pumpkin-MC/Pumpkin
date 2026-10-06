use std::{any::Any, sync::Arc};
use thiserror::Error;

use crate::{concurrent_store::LegacyStore, scheduler::TaskScheduler};

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Could not create the Wasmtime engine")]
    EngineCreationFailed(#[source] wasmtime::Error),
    #[error("Could not set up the plugin linker")]
    LinkerSetupFailed(#[source] wasmtime::Error),
    #[error("Could not identify the plugin API version (does not export pumpkin:plugin/metadata?)")]
    UnknownApiVersion(),
    #[error("Plugin is built against an unsupported version of the API: {0}")]
    UnsupportedApiVersion(String),
    #[error("The plugin does not match the expected Pumpkin API")]
    ApiMismatch(#[source] wasmtime::Error),
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
