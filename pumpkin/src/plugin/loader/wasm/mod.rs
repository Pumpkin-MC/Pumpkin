use std::{any::Any, path::Path, sync::Arc};

use wasm_host::{PluginRuntime, WasmPlugin};

use crate::plugin::{
    Context, Plugin, PluginFuture,
    loader::{PluginLoadFuture, PluginLoader, PluginUnloadFuture},
};

pub mod wasm_host;

impl Plugin for Arc<WasmPlugin> {
    fn on_load(&mut self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.as_ref()
                .on_load(context)
                .await
                .map_err(|err| err.to_string())?
        })
    }

    fn on_unload(&mut self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.as_ref()
                .on_unload(context)
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
