use std::any::Any;

#[cfg(not(target_family = "wasm"))]
use libloading::Library;

use super::{LoaderError, Path, Plugin, PluginLoader, PluginMetadata, async_trait};

#[derive(Debug)]
pub struct NativePluginLoader;

#[async_trait]
impl PluginLoader for NativePluginLoader {
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
        let path = path.to_owned();
        let library = tokio::task::spawn_blocking(move || unsafe { Library::new(&path) })
            .await
            .map_err(|e| LoaderError::RuntimeError(e.to_string()))?
            .map_err(|e| LoaderError::LibraryLoad(e.to_string()))?;

        let metadata = unsafe {
            &**library
                .get::<*const PluginMetadata>(b"METADATA")
                .map_err(|_| LoaderError::MetadataMissing)?
        };

        let plugin = unsafe {
            library
                .get::<fn() -> Box<dyn Plugin>>(b"plugin")
                .map_err(|_| LoaderError::EntrypointMissing)?
        };

        Ok((
            plugin(),
            metadata.clone(),
            Box::new(library) as Box<dyn Any + Send + Sync>,
        ))
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

    async fn unload(&self, data: Box<dyn Any + Send + Sync>) -> Result<(), LoaderError> {
        match data.downcast::<Library>() {
            Ok(_) => Ok(()),
            Err(_) => Err(LoaderError::InvalidLoaderData),
        }
    }

    /// Windows specific issue
    fn can_unload(&self) -> bool {
        !cfg!(target_os = "windows")
    }
}
