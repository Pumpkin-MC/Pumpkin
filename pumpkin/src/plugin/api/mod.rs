pub mod context;
pub mod events;

use std::sync::Arc;

use async_trait::async_trait;
pub use context::*;
pub use events::*;

/// Struct representing metadata for a plugin.
///
/// This struct contains essential information about a plugin, including its name,
/// version, authors, and a description. It is generic over a lifetime `'s` to allow
/// for string slices that are valid for the lifetime of the plugin metadata.
#[derive(Debug, Clone)]
pub struct PluginMetadata<'s> {
    /// The name of the plugin.
    pub name: &'s str,
    /// The version of the plugin.
    pub version: &'s str,
    /// The authors of the plugin.
    pub authors: &'s str,
    /// A description of the plugin.
    pub description: &'s str,
}

/// Trait representing a plugin with asynchronous lifecycle methods.
///
/// This trait defines the required methods for a plugin, including hooks for when
/// the plugin is loaded and unloaded. It is marked with `async_trait` to allow
/// for asynchronous implementations.
#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Asynchronous method called when the plugin is loaded.
    ///
    /// This method initializes the plugin within the server context.
    ///
    /// # Parameters
    /// - `_server`: Reference to the server's context.
    ///
    /// # Returns
    /// - `Ok(())` on success, or `Err(String)` on failure.
    async fn on_load(&mut self, _server: Arc<Context>) -> Result<(), String> {
        Ok(())
    }

    /// Asynchronous method called when the plugin is unloaded.
    ///
    /// This method cleans up resources when the plugin is removed from the server context.
    ///
    /// # Parameters
    /// - `_server`: Reference to the server's context.
    ///
    /// # Returns
    /// - `Ok(())` on success, or `Err(String)` on failure.
    async fn on_unload(&mut self, _server: Arc<Context>) -> Result<(), String> {
        Ok(())
    }
}
