#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(clippy::significant_drop_in_scrutinee)]
// Not warn event sending macros
#![allow(unused_labels, deprecated)]

use std::{any::Any, path::Path, sync::Arc};

use pumpkin_core::server::Server;
use pumpkin_wasm_host_common::{
    concurrent_store::{LegacySyncReentry, TokioSpawner},
    plugin::{PluginInitError, WasmPlugin},
    scheduler::TaskScheduler,
    signature::is_wasm_signed,
};
use runtime::PluginRuntime;
use tracing::warn;

use pumpkin_core::plugin::{
    Context, Plugin, PluginFuture, error,
    loader::{LoaderError, PluginLoadFuture, PluginLoader, PluginUnloadFuture},
};

mod runtime;
mod scheduler;

/// The plugin handed to the plugin manager for a loaded wasm plugin.
struct WasmPluginHandle(Arc<WasmPlugin>);

impl Plugin for WasmPluginHandle {
    fn on_load(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let headline = format!(
                "Plugin \"{}\" on_load details",
                error::inline(&context.get_metadata().name)
            );
            lifecycle_result(runtime::on_load(&self.0, context).await, &headline)
        })
    }

    fn on_unload(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            if self.0.store.state().is_terminal() {
                return Err(error::message(
                    "",
                    &["The plugin has stopped; on_unload was not delivered to it.".into()],
                    None,
                ));
            }
            let headline = format!(
                "Plugin \"{}\" on_unload details",
                error::inline(&context.get_metadata().name)
            );
            lifecycle_result(runtime::on_unload(&self.0, context).await, &headline)
        })
    }

    fn on_ipc_message(
        &self,
        sender: &str,
        message: &[u8],
    ) -> PluginFuture<'_, Result<Vec<u8>, String>> {
        let sender_own = sender.to_owned();
        let message_own = message.to_owned();
        Box::pin(async move {
            let neutral = "The receiving plugin failed to handle the message.";
            if self.0.report_stopped("IPC delivery") {
                return Err(neutral.into());
            }
            match runtime::handle_ipc_message(&self.0, &sender_own, &message_own).await {
                Ok(result) => result.map_err(|text| error::sanitize(&text)),
                Err(failure) => {
                    self.0.report_failure("IPC delivery", &failure);
                    Err(neutral.into())
                }
            }
        })
    }
}

fn lifecycle_result<T>(
    result: Result<Result<T, String>, wasmtime::Error>,
    headline: &str,
) -> Result<T, String> {
    match result {
        Ok(result) => result.map_err(|text| {
            error::message(
                "",
                &[format!(
                    "The plugin reported: \"{}\"",
                    error::sanitize(&text)
                )],
                None,
            )
        }),
        Err(failure) => {
            error::debug_details(headline, pumpkin_plugin_runtime::guest_call_error(&failure));
            Err(pumpkin_wasm_host_common::error::message(
                "",
                &pumpkin_wasm_host_common::error::causes(&failure),
                None,
                Some(&failure),
            ))
        }
    }
}

pub struct WasmPluginLoader {
    verify_signatures: bool,
    legacy_sync_reentry: LegacySyncReentry,
    task_scheduler: Arc<TaskScheduler>,
}

impl WasmPluginLoader {
    #[must_use]
    pub fn new(verify_signatures: bool) -> Self {
        if !verify_signatures {
            warn!(
                "Plugin signature verification is disabled. Only do this if you fully trust your plugins and their sources, because unsigned or tampered WASM plugins will be loaded without verification."
            );
        }
        Self {
            verify_signatures,
            legacy_sync_reentry: LegacySyncReentry::new(),
            task_scheduler: Arc::new(TaskScheduler::new()),
        }
    }
}

impl PluginLoader for WasmPluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let spawner = Arc::new(TokioSpawner::new(tokio::runtime::Handle::current()));
            let runtime = PluginRuntime::new(
                &path,
                self.legacy_sync_reentry.clone(),
                Arc::clone(&self.task_scheduler),
                spawner,
            )
            .map_err(|error| LoaderError::WasmInitializationError(Box::new(error)))?;
            let (plugin, metadata) = runtime
                .init_plugin(&path, self.verify_signatures)
                .await
                .map_err(|error| LoaderError::WasmInitializationError(Box::new(error)))?;

            Ok((
                Arc::new(WasmPluginHandle(plugin)) as Arc<dyn Plugin>,
                metadata,
                Box::new(()) as Box<dyn Any + Send + Sync>,
            ))
        })
    }

    fn report_load_error(&self, path: &Path, failure: &LoaderError) {
        let init = match failure {
            LoaderError::WasmInitializationError(source) => {
                source.downcast_ref::<PluginInitError>()
            }
            _ => None,
        };
        let operation = if matches!(init, Some(PluginInitError::ComponentNewFailed(_))) {
            "compile"
        } else {
            "load"
        };
        let file = path.file_name().map_or(path, Path::new).display();
        let headline = format!("Could not {operation} plugin file \"{file}\"; it was not loaded.");
        let backtrace = init
            .and_then(PluginInitError::wasmtime_error)
            .and_then(pumpkin_wasm_host_common::error::backtrace)
            .map(ToString::to_string);
        let help = init
            .and_then(PluginInitError::help)
            .or_else(|| failure.help());
        let text = pumpkin_wasm_host_common::error::message(
            &headline,
            &error::causes(failure, backtrace.as_deref()),
            help,
            init.and_then(PluginInitError::wasmtime_error),
        );
        tracing::error!("{text}");
        error::debug_details(&headline, failure);
    }

    fn format_lifecycle_error(&self, headline: &str, details: &str) -> String {
        // WasmPluginHandle returns a sanitized body; retain its frame indentation.
        format!("{}{details}", error::inline(headline))
    }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path.extension().unwrap_or_default();

        ext.eq_ignore_ascii_case("wasm")
    }

    fn unload(&self, _data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    fn can_unload(&self) -> bool {
        true
    }

    fn is_signed(&self, path: &Path) -> bool {
        let wasm_bytes = std::fs::read(path).unwrap_or_default();
        is_wasm_signed(&wasm_bytes)
    }

    fn on_tick(&self, server: &Arc<Server>) {
        scheduler::tick(&self.task_scheduler, server);
    }
}
