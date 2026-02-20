use crate::plugin::api::Plugin;
use pumpkin_plugin_host::{metadata::PluginMetadata, wasm_host::PluginInitError};
use std::{any::Any, path::Path, pin::Pin};
use thiserror::Error;

pub mod native;
pub mod wasm;

pub type PluginLoadFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    (Box<dyn Plugin>, PluginMetadata, Box<dyn Any + Send + Sync>),
                    LoaderError,
                >,
            > + Send
            + 'a,
    >,
>;

pub type PluginUnloadFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), LoaderError>> + Send + 'a>>;

pub trait PluginLoader: Send + Sync {
    /// Load a plugin from the specified path
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a>;

    /// Check if this loader can handle the given file
    fn can_load(&self, path: &Path) -> bool;

    fn unload(&self, data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_>;

    /// Checks if the plugin can be safely unloaded.
    fn can_unload(&self) -> bool;
}

/// Unified loader error type
#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Failed to load library: {0}")]
    LibraryLoad(String),

    #[error("Missing plugin metadata")]
    MetadataMissing,

    #[error("Missing plugin entrypoint")]
    EntrypointMissing,

    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Invalid loader data")]
    InvalidLoaderData,

    #[error(
        "Plugin was built for an incompatible API version. Please rebuild it against this Pumpkin build."
    )]
    ApiVersionMissing,

    #[error(
        "Plugin API version mismatch (plugin {plugin_version}, server {server_version}). Please rebuild it against this Pumpkin build."
    )]
    ApiVersionMismatch {
        plugin_version: u32,
        server_version: u32,
    },

    #[error("Wasm plugin initialization error: {0}")]
    WasmInitializationError(#[from] PluginInitError),
}
