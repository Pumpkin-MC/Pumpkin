use crate::plugin::api::{Plugin, PluginMetadata};
use std::{any::Any, path::Path, pin::Pin};
use thiserror::Error;

pub mod native;

pub type PluginLoadFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    (
                        Box<dyn Plugin>,
                        PluginMetadata<'static>,
                        Box<dyn Any + Send + Sync>,
                    ),
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
}
