use std::{any::Any, path::Path, sync::Arc};

use pumpkin_plugin_host::wasm_host::{
    PluginRuntime, WasmPlugin,
    state::{ContextProvider, ServerProvider},
};
use pumpkin_util::Difficulty;

use crate::{
    plugin::{
        Context, Plugin, PluginFuture,
        loader::{PluginLoadFuture, PluginLoader, PluginUnloadFuture},
    },
    server::Server,
};

struct WasmPluginContextProvider(Arc<Context>);
struct WasmPluginServerProvider(Arc<Server>);

impl ContextProvider for WasmPluginContextProvider {
    fn get_server(&self) -> Arc<dyn ServerProvider> {
        Arc::new(WasmPluginServerProvider(self.0.server.clone()))
    }
}

impl ServerProvider for WasmPluginServerProvider {
    fn get_difficulty(&self) -> Difficulty {
        self.0.get_difficulty()
    }
}

impl Plugin for WasmPlugin {
    fn on_load(&mut self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.on_load(Arc::new(WasmPluginContextProvider(context)))
                .await
                .map_err(|err| err.to_string())?
        })
    }

    fn on_unload(&mut self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.on_unload(Arc::new(WasmPluginContextProvider(context)))
                .await
                .map_err(|err| err.to_string())?
        })
    }
}

pub struct WasmPluginLoader;
impl PluginLoader for WasmPluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let runtime = PluginRuntime::new()?;

            let (plugin, metadata) = runtime.init_plugin(&path).await?;

            Ok((
                Box::new(plugin) as Box<dyn Plugin>,
                metadata,
                Box::new(()) as Box<dyn Any + Send + Sync>,
            ))
        })
    }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        ext.eq_ignore_ascii_case("wasm")
    }

    fn unload(&self, _data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async { Ok(()) })
    }

    fn can_unload(&self) -> bool {
        true
    }
}
