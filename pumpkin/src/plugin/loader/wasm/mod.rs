use std::{any::Any, path::Path, sync::Arc};

use wasm_host::{PluginRuntime, WasmPlugin};

use crate::plugin::{
    Context, Plugin, PluginFuture,
    loader::{PluginLoadFuture, PluginLoader, PluginUnloadFuture},
};

pub mod wasm_host;

impl Plugin for WasmPlugin {
    fn on_load(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            // More qualified syntax to not call the current on_load function recursively and instead call
            // WasmPlugin::on_load
            Self::on_load(self, context)
                .await
                .map_err(|err| err.to_string())
                .flatten()
        })
    }

    fn on_unload(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            Self::on_unload(self, context)
                .await
                .map_err(|err| err.to_string())
                .flatten()
        })
    }
}

pub struct WasmPluginLoader;
impl PluginLoader for WasmPluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let runtime = PluginRuntime::new(&path)?;
            let (plugin, metadata) = runtime.init_plugin(&path).await?;

            Ok((
                plugin as Arc<dyn Plugin>,
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
}
