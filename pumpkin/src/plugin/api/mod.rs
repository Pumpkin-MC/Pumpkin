pub mod context;
pub mod events;

use std::{pin::Pin, sync::Arc};

pub use context::*;
pub use events::*;

/// This type represents a future for the plugin.
pub type PluginFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Trait representing a plugin with asynchronous lifecycle methods.
///
/// This trait defines the required methods for a plugin, including hooks for when
/// the plugin is loaded and unloaded.
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
    fn on_load(&mut self, _server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
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
    fn on_unload(&mut self, _server: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }
}
