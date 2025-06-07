use async_trait::async_trait;
use futures::future::join_all;
use loader::{LoaderError, PluginLoader, native::NativePluginLoader};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::RwLock;

pub mod api;
pub mod loader;

use crate::{PERMISSION_MANAGER, server::Server};
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
    async fn handle_dyn(&self, _server: &Arc<Server>, event: &(dyn Event + Send + Sync));

    /// Asynchronously handles a blocking dynamic event.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    async fn handle_blocking_dyn(
        &self,
        _server: &Arc<Server>,
        _event: &mut (dyn Event + Send + Sync),
    );

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
    async fn handle(&self, _server: &Arc<Server>, _event: &E) {}

    /// Asynchronously handles a blocking event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    async fn handle_blocking(&self, _server: &Arc<Server>, _event: &mut E) {}
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
    async fn handle_blocking_dyn(
        &self,
        server: &Arc<Server>,
        event: &mut (dyn Event + Send + Sync),
    ) {
        if E::get_name_static() == event.get_name() {
            // Safely cast the event to the correct type and handle it.
            let event = unsafe {
                &mut *std::ptr::from_mut::<dyn std::any::Any>(event.as_any_mut()).cast::<E>()
            };
            self.handler.handle_blocking(server, event).await;
        }
    }

    /// Asynchronously handles a dynamic event.
    async fn handle_dyn(&self, server: &Arc<Server>, event: &(dyn Event + Send + Sync)) {
        if E::get_name_static() == event.get_name() {
            // Safely cast the event to the correct type and handle it.
            let event =
                unsafe { &*std::ptr::from_ref::<dyn std::any::Any>(event.as_any()).cast::<E>() };
            self.handler.handle(server, event).await;
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
    unloaded_files: HashSet<PathBuf>,
    // Self-reference for sharing with contexts
    self_ref: Option<Arc<RwLock<PluginManager>>>,
}

/// Represents a successfully loaded plugin
///
/// OS specific issues
/// - Windows: Plugin cannot be unloaded, it can be only active or not
struct LoadedPlugin {
    metadata: PluginMetadata<'static>,
    instance: Box<dyn Plugin>,
    loader: Arc<dyn PluginLoader>,
    loader_data: Box<dyn Any + Send + Sync>,
    is_active: bool,
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

    #[error("Plugin manager not initialized properly")]
    ManagerNotInitialized,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: Vec::new(),
            loaders: vec![Arc::new(NativePluginLoader)],
            server: None,
            handlers: Arc::new(RwLock::new(HashMap::new())),
            unloaded_files: HashSet::new(),
            self_ref: None,
        }
    }
}

impl PluginManager {
    /// Create a new plugin manager with default loaders
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Unload all loaded plugins
    pub async fn unload_all_plugins(&mut self) -> Result<(), ManagerError> {
        let plugin_names: Vec<&str> = self
            .plugins
            .iter()
            .filter(|p| p.is_active)
            .map(|p| p.metadata.name)
            .collect();

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(name).await {
                log::error!("Failed to unload plugin {name}: {e}");
            }
        }

        Ok(())
    }

    /// Add a new plugin loader implementation
    pub async fn add_loader(&mut self, loader: Arc<dyn PluginLoader>) {
        self.loaders.push(loader);

        // Try to load previously unloaded files with the new loader
        self.retry_unloaded_files().await;
    }

    /// Retry loading files that couldn't be loaded previously
    async fn retry_unloaded_files(&mut self) {
        let files_to_retry: Vec<PathBuf> = self.unloaded_files.iter().cloned().collect();
        for path in files_to_retry {
            if matches!(self.try_load_plugin(&path).await, Ok(())) {
                self.unloaded_files.remove(&path);
            }
        }
    }

    /// Set server reference for plugin context
    pub fn set_server(&mut self, server: Arc<Server>) {
        self.server = Some(server);
    }

    /// Set self reference for creating contexts
    pub fn set_self_ref(&mut self, self_ref: Arc<RwLock<Self>>) {
        self.self_ref = Some(self_ref);
    }

    /// Get a clone of the loaders for context use
    #[must_use]
    pub fn get_loaders(&self) -> Vec<Arc<dyn PluginLoader>> {
        self.loaders.clone()
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
                        // Remove from unloaded files if it was there
                        self.unloaded_files.remove(path);
                        return Ok(());
                    }
                    Err(e) => {
                        log::error!("Failed to load plugin {}: {}", path.display(), e);
                        return Ok(());
                    }
                }
            }
        }

        // No loader could handle this file, track it for future attempts
        self.unloaded_files.insert(path.to_path_buf());

        Err(ManagerError::PluginNotFound(
            path.to_string_lossy().to_string(),
        ))
    }

    /// Load plugin using a specific loader
    async fn load_with_loader(
        &self,
        loader: &Arc<dyn PluginLoader>,
        path: &Path,
    ) -> Result<LoadedPlugin, ManagerError> {
        let server = self
            .server
            .as_ref()
            .ok_or(ManagerError::ServerNotInitialized)?;
        let (mut instance, metadata, loader_data) = loader.load(path).await?;

        // Get a self_ref for the context or fail if not set
        let self_ref = self
            .self_ref
            .as_ref()
            .ok_or(ManagerError::ServerNotInitialized)?;

        let context = Context::new(
            metadata.clone(),
            Arc::clone(server),
            Arc::clone(&self.handlers),
            Arc::clone(self_ref),
            Arc::clone(&PERMISSION_MANAGER),
        );

        if let Err(e) = instance.on_load(&context).await {
            let data = loader_data;
            let loader = loader.clone();
            let _ = instance.on_unload(&context).await;
            tokio::spawn(async move {
                loader.unload(data).await.ok();
            });
            return Err(ManagerError::LoaderError(
                LoaderError::InitializationFailed(e),
            ));
        }

        Ok(LoadedPlugin {
            metadata,
            instance,
            loader: loader.clone(),
            loader_data,
            is_active: true,
        })
    }

    /// Checks if plugin active
    #[must_use]
    pub fn is_plugin_active(&self, name: &str) -> bool {
        self.plugins
            .iter()
            .any(|p| p.metadata.name == name && p.is_active)
    }

    /// Get list of active plugins
    #[must_use]
    pub fn active_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins
            .iter()
            .filter(|p| p.is_active)
            .map(|p| &p.metadata)
            .collect()
    }

    /// Checks if plugin loaded
    #[must_use]
    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.plugins.iter().any(|p| p.metadata.name == name)
    }

    /// Get list of loaded plugins
    #[must_use]
    pub fn loaded_plugins(&self) -> Vec<&PluginMetadata> {
        self.plugins.iter().map(|p| &p.metadata).collect()
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), ManagerError> {
        let index = self
            .plugins
            .iter()
            .position(|p| p.metadata.name == name)
            .ok_or_else(|| ManagerError::PluginNotFound(name.to_string()))?;

        let mut plugin = self.plugins.remove(index);
        let server = self
            .server
            .as_ref()
            .ok_or(ManagerError::ServerNotInitialized)?;

        // Get a self_ref for the context or fail if not set
        let self_ref = self
            .self_ref
            .as_ref()
            .ok_or(ManagerError::ServerNotInitialized)?;

        let context = Context::new(
            plugin.metadata.clone(),
            Arc::clone(server),
            Arc::clone(&self.handlers),
            Arc::clone(self_ref),
            Arc::clone(&PERMISSION_MANAGER),
        );

        plugin.instance.on_unload(&context).await.ok();

        if plugin.loader.can_unload() {
            plugin.loader.unload(plugin.loader_data).await?;
        } else {
            plugin.is_active = false;
            self.plugins.push(plugin);
        }

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
        if let Some(server) = &self.server {
            let handlers = self.handlers.read().await;
            if let Some(handlers) = handlers.get(&E::get_name_static()) {
                let (blocking, non_blocking): (Vec<_>, Vec<_>) =
                    handlers.iter().partition(|h| h.is_blocking());

                // Process blocking handlers first
                for handler in blocking {
                    handler.handle_blocking_dyn(server, &mut event).await;
                }

                // Process non-blocking handlers
                join_all(
                    non_blocking
                        .into_iter()
                        .map(|h| h.handle_dyn(server, &event)),
                )
                .await;
            }
        }
        event
    }
}
