use crate::{
    plugin::{PluginMetadata, api::Plugin},
    server::Server,
};
use std::{any::Any, path::Path, pin::Pin, sync::Arc};
use thiserror::Error;

pub mod native;

pub type PluginLoadFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<
                    (Arc<dyn Plugin>, PluginMetadata, Box<dyn Any + Send + Sync>),
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

    /// Logs the failed load at ERROR, with technical details at DEBUG.
    fn report_load_error(&self, path: &Path, error: &LoaderError) {
        let file = path.file_name().map_or(path, Path::new).display();
        let headline = format!("Could not load plugin file \"{file}\".");
        let text =
            super::error::message(&headline, &super::error::causes(error, None), error.help());
        tracing::error!("{text}");
        super::error::debug_details(&headline, error);
    }

    /// Formats a lifecycle error with the plugin's name and version.
    fn format_lifecycle_error(
        &self,
        metadata: &PluginMetadata,
        outcome: &str,
        details: &str,
    ) -> String {
        super::error::message(
            &super::error::lifecycle_headline(metadata, outcome),
            &details
                .lines()
                .map(super::error::sanitize)
                .collect::<Vec<_>>(),
            None,
        )
    }

    /// Check if this loader can handle the given file
    fn can_load(&self, path: &Path) -> bool;

    fn unload(&self, data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_>;

    /// Checks if the plugin can be safely unloaded.
    fn can_unload(&self) -> bool;

    /// Whether the plugin file at `path` carries a valid signature. Loaders without signing accept every file.
    fn is_signed(&self, _path: &Path) -> bool {
        true
    }

    /// Called once per server tick for loaders that run work of their own, like scheduled plugin tasks.
    fn on_tick(&self, _server: &Arc<Server>) {}
}

/// Unified loader error type
#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Could not load the native plugin library")]
    LibraryLoad(#[source] libloading::Error),

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

    #[error("Native plugin API version is missing")]
    ApiVersionMissing,

    #[error("Plugin API version mismatch (plugin {plugin_version}, server {server_version})")]
    ApiVersionMismatch {
        plugin_version: u32,
        server_version: u32,
    },

    #[error(transparent)]
    WasmInitializationError(Box<dyn std::error::Error + Send + Sync>),
}

impl LoaderError {
    /// Returns a hint when the error identifies a specific action.
    #[must_use]
    pub const fn help(&self) -> Option<&'static str> {
        match self {
            Self::ApiVersionMissing | Self::ApiVersionMismatch { .. } => Some(
                "Install a compatible native plugin, or rebuild it against this Pumpkin build.",
            ),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoaderError;

    #[test]
    fn native_api_help_does_not_depend_on_error_text() {
        assert!(LoaderError::ApiVersionMissing.help().is_some());
        assert!(
            LoaderError::ApiVersionMismatch {
                plugin_version: 1,
                server_version: 2
            }
            .help()
            .is_some()
        );
        for error in [
            LoaderError::MetadataMissing,
            LoaderError::EntrypointMissing,
            LoaderError::InvalidLoaderData,
            LoaderError::InitializationFailed("API mismatch".into()),
            LoaderError::RuntimeError("Permission denied".into()),
            LoaderError::WasmInitializationError(Box::new(std::io::Error::other("API mismatch"))),
        ] {
            assert!(error.help().is_none());
        }
    }
}
