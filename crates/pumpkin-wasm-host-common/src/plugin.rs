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
    #[error("The plugin targets API {0}; this server supports {1}")]
    UnsupportedApiVersion(String, &'static str),
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

pub const SUPPORTED_PLUGIN_APIS: [(&str, PluginApiVersion); 2] = [
    ("0.1", PluginApiVersion::V0_1),
    ("0.2", PluginApiVersion::V0_2),
];

impl PluginInitError {
    /// An action supported by this error category.
    #[must_use]
    pub fn help(&self) -> Option<&'static str> {
        match self {
            Self::UnsupportedApiVersion(..) => {
                Some("Install a plugin release targeting a supported Pumpkin API.")
            }
            Self::ApiMismatch(_) => {
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

    /// The original Wasmtime error, when this failure came from Wasmtime.
    #[must_use]
    pub const fn wasmtime_error(&self) -> Option<&wasmtime::Error> {
        match self {
            Self::EngineCreationFailed(error)
            | Self::LinkerSetupFailed(error)
            | Self::ApiMismatch(error)
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

#[derive(Copy, Clone)]
pub enum PluginApiVersion {
    V0_1,
    V0_2,
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
    /// Reports a callback that could not reach a stopped store.
    #[must_use]
    pub fn report_stopped(&self, operation: &str) -> bool {
        if !self.store.state().is_terminal() {
            return false;
        }
        tracing::error!(
            "Plugin \"{}\" ({}) has stopped; {} was not delivered to it.",
            pumpkin_core::plugin::error::inline(&self.name),
            pumpkin_core::plugin::error::inline(&self.version),
            pumpkin_core::plugin::error::inline(operation)
        );
        true
    }

    /// Reports a Wasmtime callback failure with its original technical details.
    pub fn report_failure(&self, operation: &str, failure: &wasmtime::Error) {
        use pumpkin_core::plugin::error;
        let headline = format!(
            "Plugin \"{}\" ({}) failed during {}.",
            error::inline(&self.name),
            error::inline(&self.version),
            error::inline(operation)
        );
        let stopped = self.store.state().is_terminal();
        let help = crate::error::help(failure).or_else(|| {
            stopped.then_some("Unload and load the plugin again, or restart the server, to retry.")
        });
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
    use super::PluginInitError;
    use std::io::{Error, ErrorKind};

    #[test]
    fn help_is_limited_to_actionable_typed_failures() {
        assert!(
            PluginInitError::UnsupportedApiVersion("0.3.0".into(), "0.1 and 0.2")
                .help()
                .is_some()
        );
        assert!(
            PluginInitError::ApiMismatch(wasmtime::Error::msg("type mismatch"))
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
}
