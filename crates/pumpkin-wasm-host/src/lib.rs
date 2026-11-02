#![deny(clippy::unwrap_used)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(clippy::significant_drop_in_scrutinee)]
// Not warn event sending macros
#![allow(unused_labels, deprecated)]

use std::{any::Any, path::Path, sync::Arc};

use pumpkin_core::server::Server;
use pumpkin_wasm_host_common::{
    concurrent_store::{LegacySyncReentry, TokioSpawner},
    plugin::WasmPlugin,
    scheduler::TaskScheduler,
    signature::is_wasm_signed,
};
use runtime::PluginRuntime;
use tracing::warn;

use pumpkin_core::plugin::{
    Context, Plugin, PluginFuture,
    loader::{LoaderError, PluginLoadFuture, PluginLoader, PluginUnloadFuture},
};

mod runtime;
mod scheduler;

/// The plugin handed to the plugin manager for a loaded wasm plugin.
struct WasmPluginHandle(Arc<WasmPlugin>);

impl Plugin for WasmPluginHandle {
    fn on_load(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            runtime::on_load(&self.0, context)
                .await
                .map_err(|err| err.to_string())
                .flatten()
        })
    }

    fn on_unload(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            runtime::on_unload(&self.0, context)
                .await
                .map_err(|err| err.to_string())
                .flatten()
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
            runtime::handle_ipc_message(&self.0, &sender_own, &message_own)
                .await
                .map_err(|err| err.to_string())
                .flatten()
        })
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
