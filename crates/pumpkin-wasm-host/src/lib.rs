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

    /// Creates a [`NativeWitPluginLoader`] sharing the same cross-plugin reentry policy.
    #[must_use]
    pub fn native_wit_loader(&self) -> NativeWitPluginLoader {
        NativeWitPluginLoader {
            legacy_sync_reentry: self.legacy_sync_reentry.clone(),
            task_scheduler: Arc::new(TaskScheduler::new()),
        }
    }
}

/// Retained by the plugin manager for native WIT plugins.
///
/// Holding a clone of the `libloading::Library` here ensures that on Windows (where
/// `can_unload()` returns `false` and the manager drops the plugin instance while keeping loader
/// data), the DLL is not unloaded via `FreeLibrary`. On platforms that support unloading, the
/// manager calls `unload(data)` which drops this handle alongside the plugin instance.
struct NativeWitLoaderData {
    language: String,
    _library: Option<Arc<pumpkin_native_wit::libloading::Library>>,
}

/// Loads native (`cdylib`) shared libraries compiled against Pumpkin's WIT world via `wit-bindgen`.
pub struct NativeWitPluginLoader {
    legacy_sync_reentry: LegacySyncReentry,
    task_scheduler: Arc<TaskScheduler>,
}

impl Default for NativeWitPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeWitPluginLoader {
    #[must_use]
    pub fn new() -> Self {
        Self {
            legacy_sync_reentry: LegacySyncReentry::new(),
            task_scheduler: Arc::new(TaskScheduler::new()),
        }
    }
}

fn is_native_library_path(path: &Path) -> bool {
    let ext = path.extension().unwrap_or_default();
    if cfg!(target_os = "windows") {
        ext.eq_ignore_ascii_case("dll")
    } else if cfg!(target_os = "macos") {
        ext.eq_ignore_ascii_case("dylib")
    } else {
        ext.eq_ignore_ascii_case("so")
    }
}

impl PluginLoader for NativeWitPluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();
            let spawner = Arc::new(TokioSpawner::new(tokio::runtime::Handle::current()));
            let (plugin, metadata) = runtime::init_native_plugin(
                &path,
                &self.legacy_sync_reentry,
                Arc::clone(&self.task_scheduler),
                spawner,
            )
            .await
            .map_err(|error| LoaderError::WasmInitializationError(Box::new(error)))?;

            let language = plugin
                .binding_language
                .clone()
                .unwrap_or_else(|| "native-wit".to_string());
            let library = runtime::native_library(&plugin);

            Ok((
                Arc::new(WasmPluginHandle(plugin)) as Arc<dyn Plugin>,
                metadata,
                Box::new(NativeWitLoaderData {
                    language,
                    _library: library,
                }) as Box<dyn Any + Send + Sync>,
            ))
        })
    }

    fn can_load(&self, path: &Path) -> bool {
        is_native_library_path(path)
            && pumpkin_native_wit::detect_native_wit_world_version(path).is_some()
    }

    fn unload(&self, _data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    fn can_unload(&self) -> bool {
        !cfg!(target_os = "windows")
    }

    fn on_tick(&self, server: &Arc<Server>) {
        scheduler::tick(&self.task_scheduler, server);
    }

    fn binding_language(&self, data: &(dyn Any + Send + Sync)) -> Option<String> {
        data.downcast_ref::<NativeWitLoaderData>()
            .map(|loader_data| loader_data.language.clone())
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

            let language = plugin
                .binding_language
                .clone()
                .unwrap_or_else(|| "wasm".to_string());

            Ok((
                Arc::new(WasmPluginHandle(plugin)) as Arc<dyn Plugin>,
                metadata,
                Box::new(language) as Box<dyn Any + Send + Sync>,
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

    fn binding_language(&self, data: &(dyn Any + Send + Sync)) -> Option<String> {
        data.downcast_ref::<String>().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::WasmPluginLoader;
    use pumpkin_core::plugin::loader::PluginLoader;

    #[test]
    fn native_wit_loader_ignores_wasm_and_non_wit_shared_libraries() {
        let dir = tempfile::tempdir().expect("tempdir");
        let wasm_path = dir.path().join("demo.wasm");
        std::fs::write(&wasm_path, b"not a wasm component").expect("write wasm");

        let native_ext = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };
        let non_wit_lib = dir.path().join(format!("legacy.{native_ext}"));
        std::fs::write(&non_wit_lib, b"not an elf/pe/macho library").expect("write dummy lib");

        let wasm_loader = WasmPluginLoader::new(false);
        let native_wit_loader = wasm_loader.native_wit_loader();

        assert!(wasm_loader.can_load(&wasm_path));
        assert!(!wasm_loader.can_load(&non_wit_lib));
        assert!(!native_wit_loader.can_load(&wasm_path));
        assert!(!native_wit_loader.can_load(&non_wit_lib));
    }
}
