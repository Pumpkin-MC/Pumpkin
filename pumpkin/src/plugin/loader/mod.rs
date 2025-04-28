use crate::plugin::api::{Plugin, PluginMetadata};
use async_trait::async_trait;
use std::{any::Any, path::Path};
use thiserror::Error;

pub mod native;

/// Common trait for all plugin loaders
#[async_trait]
pub trait PluginLoader: Send + Sync {
    /// Load a plugin from the specified path
    async fn load(
        &self,
        path: &Path,
    ) -> Result<
        (
            Box<dyn Plugin>,
            PluginMetadata<'static>,
            Box<dyn Any + Send + Sync>,
        ),
        LoaderError,
    >;

    /// Check if this loader can handle the given file
    fn can_load(&self, path: &Path) -> bool;

    async fn unload(&self, data: Box<dyn Any + Send + Sync>) -> Result<(), LoaderError>;

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
