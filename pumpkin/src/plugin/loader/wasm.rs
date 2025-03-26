use super::{LoaderError, PluginLoader};
use crate::plugin::api::{Plugin, PluginMetadata};
use async_trait::async_trait;
use bincode::config;
use std::any::Any;
use std::path::Path;
use wasmtime::{Config, Engine, Instance, Module, Store};

/// WASM Plugin Loader using Wasmtime
pub struct WasmPluginLoader;

#[async_trait]
impl PluginLoader for WasmPluginLoader {
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
    > {
        // Configure Wasmtime engine with async support
        let mut config = Config::new();
        config.async_support(true);

        let engine = Engine::new(&config).map_err(|e| LoaderError::LibraryLoad(e.to_string()))?;

        // Load WASM module from file
        let module = Module::from_file(&engine, path)
            .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?;

        // Create store and instantiate module
        let mut store = Store::new(&engine, ());
        let instance = Instance::new_async(&mut store, &module, &[])
            .await
            .map_err(|e| LoaderError::RuntimeError(e.to_string()))?;

        // Extract metadata from WASM module
        let metadata = extract_metadata(&instance, &mut store).await?;

        // Create WASM plugin wrapper
        let plugin = WasmPlugin { store, instance };

        // Store engine and module for unloading
        let loader_data = Box::new((engine, module));

        Ok((Box::new(plugin), metadata, loader_data))
    }

    fn can_load(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("wasm"))
            .unwrap_or(false)
    }

    async fn unload(&self, _data: Box<dyn Any + Send + Sync>) -> Result<(), LoaderError> {
        // WASM modules are unloaded automatically when references are dropped
        Ok(())
    }
}

/// Helper to extract metadata from WASM module
async fn extract_metadata(
    instance: &Instance,
    store: &mut Store<()>,
) -> Result<PluginMetadata<'static>, LoaderError> {
    // Get metadata export function
    let get_metadata = instance
        .get_typed_func::<(), (i32, u32)>(&mut *store, "get_metadata")
        .map_err(|_| LoaderError::MetadataMissing)?;

    // Call function to get metadata pointer/length
    let (ptr, len) = get_metadata
        .call_async(&mut *store, ())
        .await
        .map_err(|e| LoaderError::RuntimeError(e.to_string()))?;

    // Access module memory
    let memory = instance
        .get_memory(&mut *store, "memory")
        .ok_or(LoaderError::MetadataMissing)?;

    // Read bytes from WASM memory
    let mut bytes = vec![0u8; len as usize];
    memory
        .read(store, ptr as usize, &mut bytes)
        .map_err(|_| LoaderError::MetadataMissing)?;

    // Deserialize metadata
    bincode::decode_from_slice(&bytes, config::standard())
        .map_err(|_| LoaderError::MetadataMissing)
        .map(|r| r.0)
}

/// WASM Plugin wrapper implementing Plugin trait
struct WasmPlugin {
    store: Store<()>,
    instance: Instance,
}

#[async_trait]
impl Plugin for WasmPlugin {
    async fn on_load(&mut self, _context: &crate::plugin::Context) -> Result<(), String> {
        // Call WASM plugin's on_load function
        let func = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, "on_load")
            .map_err(|e| e.to_string())?;

        func.call_async(&mut self.store, ())
            .await
            .map_err(|e| e.to_string())
    }

    async fn on_unload(&mut self, _context: &crate::plugin::Context) -> Result<(), String> {
        // Call WASM plugin's on_unload function
        let func = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, "on_unload")
            .map_err(|e| e.to_string())?;

        func.call_async(&mut self.store, ())
            .await
            .map_err(|e| e.to_string())
    }
}
