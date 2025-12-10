use std::any::Any;

use libloading::Library;

use crate::plugin::loader::{PluginLoadFuture, PluginUnloadFuture};

use super::{LoaderError, Path, Plugin, PluginLoader, PluginMetadata};

pub struct NativePluginLoader;

impl PluginLoader for NativePluginLoader {
    fn load<'a>(&'a self, path: &'a Path) -> PluginLoadFuture<'a> {
        Box::pin(async {
            let path = path.to_owned();

            let library = unsafe { Library::new(&path) }
                .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?;

            // 2. Extract Metadata (METADATA)
            let metadata = unsafe {
                &**library
                    .get::<*const PluginMetadata>(b"METADATA")
                    .map_err(|_| LoaderError::MetadataMissing)?
            };

            // 3. Extract Plugin Factory (plugin)
            let plugin_factory = unsafe {
                library
                    .get::<fn() -> Box<dyn Plugin>>(b"plugin")
                    .map_err(|_| LoaderError::EntrypointMissing)?
            };

            Ok((
                plugin_factory(),
                metadata.clone(),
                Box::new(library) as Box<dyn Any + Send + Sync>,
            ))
        })
    }

    fn can_load(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

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
