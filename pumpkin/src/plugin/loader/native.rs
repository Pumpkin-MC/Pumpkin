use std::any::Any;

use libloading::Library;

use crate::plugin::{
    PLUGIN_API_VERSION,
    loader::{PluginLoadFuture, PluginUnloadFuture},
};

use super::{LoaderError, Path, Plugin, PluginLoader, PluginMetadata};

pub struct NativePluginLoader;

/// Upper bound for plausible metadata text field lengths, in bytes.
const MAX_METADATA_TEXT_LEN: usize = 4096;
/// Upper bound for plausible metadata list lengths, in entries.
const MAX_METADATA_LIST_LEN: usize = 1024;

/// Best-effort check that metadata read from a plugin binary is plausible.
///
/// `METADATA` is plain Rust data whose layout is not stable across
/// toolchains. If the plugin was built with a different Rust version or
/// against a different Pumpkin build, the field lengths read here can be
/// garbage even when the API version check passes, and cloning or logging
/// such metadata aborts the server with an enormous allocation (see issue
/// #2434). Only lengths and capacities are inspected here; the potentially
/// invalid data pointers are never dereferenced.
fn is_metadata_plausible(metadata: &PluginMetadata) -> bool {
    let text_plausible =
        |text: &String| text.len() <= MAX_METADATA_TEXT_LEN && text.len() <= text.capacity();
    let list_plausible =
        |list: &Vec<String>| list.len() <= MAX_METADATA_LIST_LEN && list.len() <= list.capacity();

    !metadata.name.is_empty()
        && text_plausible(&metadata.name)
        && text_plausible(&metadata.version)
        && text_plausible(&metadata.description)
        && list_plausible(&metadata.authors)
        && list_plausible(&metadata.dependencies)
        && list_plausible(&metadata.permissions)
}

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
            let metadata = unsafe {
                &**library
                    .get::<*const PluginMetadata>(b"METADATA")
                    .map_err(|_| LoaderError::MetadataMissing)?
            };

            if !is_metadata_plausible(metadata) {
                return Err(LoaderError::MetadataCorrupt);
            }

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
