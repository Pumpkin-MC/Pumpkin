use std::{
    any::Any,
    sync::{Arc, LazyLock},
};

use libloading::Library;

use crate::plugin::{
    PLUGIN_API_VERSION,
    loader::{PluginLoadFuture, PluginUnloadFuture},
};

use super::{LoaderError, Path, Plugin, PluginLoader, PluginMetadata};

pub struct NativePluginLoader;

impl PluginLoader for NativePluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let library = unsafe { Library::new(&path) }
                .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?;

            // Ensure this plugin was built against a compatible Pumpkin plugin API version
            let plugin_api_version = unsafe {
                match library.get::<*const u32>(b"PUMPKIN_API_VERSION") {
                    Ok(symbol) => **symbol,
                    Err(_) => return Err(LoaderError::ApiVersionMissing),
                }
            };

            if plugin_api_version != PLUGIN_API_VERSION {
                return Err(LoaderError::ApiVersionMismatch {
                    plugin_version: plugin_api_version,
                    server_version: PLUGIN_API_VERSION,
                });
            }

            // 2. Extract Metadata (METADATA)
            // `#[plugin_impl]` exports this as a `LazyLock`, since `PluginMetadata`
            // owns its strings and can't be built in a const.
            let metadata = unsafe {
                let metadata = library
                    .get::<*const LazyLock<PluginMetadata>>(b"METADATA")
                    .map_err(|_| LoaderError::MetadataMissing)?;
                (**metadata).clone()
            };

            // 3. Extract Plugin Factory (plugin)
            let plugin_factory = unsafe {
                library
                    .get::<fn() -> Box<dyn Plugin>>(b"plugin")
                    .map_err(|_| LoaderError::EntrypointMissing)?
            };

            Ok((
                Arc::from(plugin_factory()),
                metadata,
                Box::new(library) as Box<dyn Any + Send + Sync>,
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

    fn unload(&self, data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async {
            data.downcast::<Library>()
                .map_or(Err(LoaderError::InvalidLoaderData), |library| {
                    drop(library);
                    Ok(())
                })
        })
    }

    /// Windows specific issue: Windows locks DLLs, so we must indicate they cannot be unloaded.
    fn can_unload(&self) -> bool {
        !cfg!(target_os = "windows")
    }
}
