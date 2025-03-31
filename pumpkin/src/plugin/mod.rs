use async_trait::async_trait;
use loader::{LoaderError, PluginLoader, native::NativePluginLoader};
use std::{any::Any, collections::HashMap, path::Path, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

pub mod api;
pub mod loader;

use crate::server::Server;
pub use api::*;

/// A trait for handling events dynamically.
///
/// This trait allows for handling events of any type that implements the `Event` trait.
#[async_trait]
pub trait DynEventHandler: Send + Sync {
    /// Asynchronously handles a dynamic event.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    async fn handle_dyn(&self, event: &(dyn Event + Send + Sync));

    /// Asynchronously handles a blocking dynamic event.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    async fn handle_blocking_dyn(&self, _event: &mut (dyn Event + Send + Sync));

    /// Checks if the event handler is blocking.
    ///
    /// # Returns
    /// A boolean indicating whether the handler is blocking.
    fn is_blocking(&self) -> bool;

    /// Retrieves the priority of the event handler.
    ///
    /// # Returns
    /// The priority of the event handler.
    fn get_priority(&self) -> EventPriority;
}

/// A trait for handling specific events.
///
/// This trait allows for handling events of a specific type that implements the `Event` trait.
#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    /// Asynchronously handles an event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    async fn handle(&self, _event: &E) {
        unimplemented!();
    }

    /// Asynchronously handles a blocking event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    async fn handle_blocking(&self, _event: &mut E) {
        unimplemented!();
    }
}

/// A struct representing a typed event handler.
///
/// This struct holds a reference to an event handler, its priority, and whether it is blocking.
struct TypedEventHandler<E, H>
where
    E: Event + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    handler: Arc<H>,
    priority: EventPriority,
    blocking: bool,
    _phantom: std::marker::PhantomData<E>,
}

#[async_trait]
impl<E, H> DynEventHandler for TypedEventHandler<E, H>
where
    E: Event + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    /// Asynchronously handles a blocking dynamic event.
    async fn handle_blocking_dyn(&self, event: &mut (dyn Event + Send + Sync)) {
        if E::get_name_static() == event.get_name() {
            // Safely cast the event to the correct type and handle it.
            let event = unsafe {
                &mut *std::ptr::from_mut::<dyn std::any::Any>(event.as_any_mut()).cast::<E>()
            };
            self.handler.handle_blocking(event).await;
        }
    }

    /// Asynchronously handles a dynamic event.
    async fn handle_dyn(&self, event: &(dyn Event + Send + Sync)) {
        if E::get_name_static() == event.get_name() {
            // Safely cast the event to the correct type and handle it.
            let event =
                unsafe { &*std::ptr::from_ref::<dyn std::any::Any>(event.as_any()).cast::<E>() };
            self.handler.handle(event).await;
        }
    }

    /// Checks if the handler is blocking.
    fn is_blocking(&self) -> bool {
        self.blocking
    }

    /// Retrieves the priority of the handler.
    fn get_priority(&self) -> EventPriority {
        self.priority.clone()
    }
}

/// A type alias for a map of event handlers, where the key is a static string
/// and the value is a vector of dynamic event handlers.
type HandlerMap = HashMap<&'static str, Vec<Box<dyn DynEventHandler>>>;

/// Core plugin management system
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    loaders: Vec<Arc<dyn PluginLoader>>,
    server: Option<Arc<Server>>,
    handlers: Arc<RwLock<HandlerMap>>,
}

/// Represents a successfully loaded plugin
struct LoadedPlugin {
    metadata: PluginMetadata<'static>,
    instance: Box<dyn Plugin>,
    loader: Arc<dyn PluginLoader>,
    loader_data: Box<dyn Any + Send + Sync>,
}

/// Error types for plugin management
#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Server not initialized")]
    ServerNotInitialized,

    #[error("Plugin not found: {0}")]
    PluginNotFound(String),

    #[error("Loader error: {0}")]
    LoaderError(#[from] LoaderError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            loaders: vec![Arc::new(NativePluginLoader)],
            server: None,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl PluginManager {
    /// Create a new plugin manager with default loaders
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new plugin loader implementation
    pub fn add_loader(&mut self, loader: Arc<dyn PluginLoader>) {
        self.loaders.push(loader);
    }

    /// Set server reference for plugin context
    pub fn set_server(&mut self, server: Arc<Server>) {
        self.server = Some(server);
    }

    /// Load all plugins from the plugin directory
    pub async fn load_plugins(&mut self) -> Result<(), ManagerError> {
        const PLUGIN_DIR: &str = "./plugins";
        let path = Path::new(PLUGIN_DIR);

        if !path.exists() {
            std::fs::create_dir(path)?;
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            self.try_load_plugin(&path).await?;
        }

        Ok(())
    }

    /// Attempt to load a single plugin file
    pub async fn try_load_plugin(&mut self, path: &Path) -> Result<(), ManagerError> {
        for loader in &self.loaders {
            if loader.can_load(path) {
                match self.load_with_loader(loader, path).await {
                    Ok(plugin) => {
                        self.plugins.push(plugin);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Failed to load plugin {}: {}", path.display(), e);
                        return Ok(());
                    }
                }
            }
        }
        Err(ManagerError::PluginNotFound(
            path.to_string_lossy().to_string(),
        ))
    }

    /// Load plugin using a specific loader
    async fn load_with_loader(
        &self,
        loader: &Arc<dyn PluginLoader>,
        path: &Path,
    ) -> Result<LoadedPlugin, LoaderError> {
        let server = self
            .server
            .as_ref()
            .ok_or(LoaderError::ServerNotInitialized)?;
        let (mut instance, metadata, loader_data) = loader.load(path).await?;

        let context = Context::new(
            metadata.clone(),
            Arc::clone(server),
            Arc::clone(&self.handlers),
        );

        if let Err(e) = instance.on_load(&context).await {
            let data = loader_data;
            let loader = loader.clone();
            let _ = instance.on_unload(&context).await;
            tokio::spawn(async move {
                loader.unload(data).await.ok();
            });
            return Err(LoaderError::InitializationFailed(e));
        }

        Ok(LoadedPlugin {
            metadata,
            instance,
            loader: loader.clone(),
            loader_data,
        })
    }

    /// Get list of loaded plugins
    #[must_use]
    pub fn loaded_plugins(&self) -> Vec<&PluginMetadata<'static>> {
        self.plugins.iter().map(|p| &p.metadata).collect()
    }

    #[must_use]
    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.plugins.iter().any(|p| p.metadata.name == name)
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), ManagerError> {
        let index = self
            .plugins
            .iter()
            .position(|p| p.metadata.name == name)
            .ok_or_else(|| ManagerError::PluginNotFound(name.to_string()))?;

        let mut plugin = self.plugins.swap_remove(index);
        let server = self
            .server
            .as_ref()
            .ok_or(ManagerError::ServerNotInitialized)?;

        let context = Context::new(
            plugin.metadata.clone(),
            Arc::clone(server),
            Arc::clone(&self.handlers),
        );

        plugin.instance.on_unload(&context).await.ok();
        plugin.loader.unload(plugin.loader_data).await?;

        Ok(())
    }

    /// Register an event handler
    pub async fn register<E, H>(&self, handler: Arc<H>, priority: EventPriority, blocking: bool)
    where
        E: Event + Send + Sync + 'static,
        H: EventHandler<E> + 'static,
    {
        let mut handlers = self.handlers.write().await;
        let typed_handler = TypedEventHandler {
            handler,
            priority,
            blocking,
            _phantom: std::marker::PhantomData,
        };

        handlers
            .entry(E::get_name_static())
            .or_default()
            .push(Box::new(typed_handler));
    }

    /// Fire an event to all registered handlers
    pub async fn fire<E: Event + Send + Sync + 'static>(&self, mut event: E) -> E {
        let handlers = self.handlers.read().await;
        if let Some(handlers) = handlers.get(&E::get_name_static()) {
            let (blocking, non_blocking): (Vec<_>, Vec<_>) =
                handlers.iter().partition(|h| h.is_blocking());

            // Process blocking handlers first
            for handler in blocking {
                handler.handle_blocking_dyn(&mut event).await;
            }

            // Process non-blocking handlers
            for handler in non_blocking {
                handler.handle_dyn(&event).await;
            }
        }
        event
    }
}
