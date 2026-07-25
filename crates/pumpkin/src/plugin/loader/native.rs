use std::{any::Any, sync::Arc, sync::LazyLock};

use tokio::sync::Mutex;
use wasmtime::{Engine, Store};

use crate::plugin::{
    PluginMetadata,
    loader::{
        PluginLoadFuture, PluginUnloadFuture,
        wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState, wit::v0_1::native},
    },
};

use super::{LoaderError, Path, Plugin, PluginLoader};

static STORE_ENGINE: LazyLock<Engine> = LazyLock::new(Engine::default);

pub struct NativePluginLoader;

impl PluginLoader for NativePluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let plugin = native::AnyPlugin::Native(
                // SAFETY: `Plugin::new` dlopen's `path` and looks up the symbols the
                // `plugin` WIT world defines. Loading a shared library runs its
                // initializers, so the caller vouches for the file being a trusted
                // cdylib built against this WIT world; every symbol lookup inside is
                // checked and reported as an error rather than assumed to exist.
                unsafe { native::Plugin::new(&path) }
                    .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?,
            );

            let mut store = Store::new(&STORE_ENGINE, PluginHostState::new());

            plugin
                .call_init_plugin(&mut store)
                .await
                .map_err(|e| LoaderError::InitializationFailed(e.to_string()))?;

            let metadata = plugin
                .call_get_metadata(&mut store)
                .await
                .map_err(|e| LoaderError::InitializationFailed(e.to_string()))?;

            let metadata = PluginMetadata {
                name: metadata.name,
                version: metadata.version,
                authors: metadata.authors,
                description: metadata.description,
                dependencies: metadata.dependencies,
                permissions: metadata.permissions,
            };

            store
                .data_mut()
                .permissions
                .clone_from(&metadata.permissions);

            let plugin = Arc::new(WasmPlugin {
                plugin_instance: PluginInstance::V0_1(plugin),
                store: Mutex::new(store),
            });
            plugin.store.lock().await.data_mut().plugin = Some(Arc::downgrade(&plugin));

            Ok((
                plugin as Arc<dyn Plugin>,
                metadata,
                Box::new(()) as Box<dyn Any + Send + Sync>,
            ))
        })
    }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path.extension().unwrap_or_default();

        if cfg!(target_os = "windows") {
            ext.eq_ignore_ascii_case("dll")
        } else if cfg!(target_os = "macos") {
            ext.eq_ignore_ascii_case("dylib")
        } else {
            ext.eq_ignore_ascii_case("so")
        }
    }

    fn unload(&self, _data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    /// Windows specific issue: Windows locks DLLs, so we must indicate they cannot be unloaded.
    fn can_unload(&self) -> bool {
        !cfg!(target_os = "windows")
    }
}
