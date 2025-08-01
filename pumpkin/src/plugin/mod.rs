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
use tokio::sync::{Notify, RwLock};

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

/// Plugin loading state
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Loading,
    Loaded,
    Failed(String),
}

/// Core plugin management system
pub struct PluginManager {
    plugins: RwLock<Vec<LoadedPlugin>>,
    loaders: RwLock<Vec<Arc<dyn PluginLoader>>>,
    server: RwLock<Option<Arc<Server>>>,
    handlers: Arc<RwLock<HandlerMap>>,
    unloaded_files: RwLock<HashSet<PathBuf>>,
    // Self-reference for sharing with contexts
    self_ref: RwLock<Option<Arc<PluginManager>>>,
    services: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    // Plugin state tracking
    plugin_states: RwLock<HashMap<String, PluginState>>,
    // Notification for plugin state changes
    state_notify: Arc<Notify>,
}

/// Represents a successfully loaded plugin
///
/// OS specific issues
/// - Windows: Plugin cannot be unloaded, it can be only active or not
struct LoadedPlugin {
    metadata: PluginMetadata<'static>,
    instance: Option<Box<dyn Plugin>>,
    loader: Arc<dyn PluginLoader>,
    loader_data: Option<Box<dyn Any + Send + Sync>>,
    is_active: bool,
    context: Arc<Context>,
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
            plugins: RwLock::new(Vec::new()),
            loaders: RwLock::new(vec![Arc::new(NativePluginLoader)]),
            server: RwLock::new(None),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            unloaded_files: RwLock::new(HashSet::new()),
            self_ref: RwLock::new(None),
            services: Arc::new(RwLock::new(HashMap::new())),
            plugin_states: RwLock::new(HashMap::new()),
            state_notify: Arc::new(Notify::new()),
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
    pub async fn unload_all_plugins(&self) -> Result<(), ManagerError> {
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().await;
            plugins
                .iter()
                .filter(|p| p.is_active)
                .map(|p| p.metadata.name.to_string())
                .collect()
        };

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                log::error!("Failed to unload plugin {name}: {e}");
            }
        }

        Ok(())
    }

    /// Add a new plugin loader implementation
    pub async fn add_loader(&self, loader: Arc<dyn PluginLoader>) {
        self.loaders.write().await.push(loader);

        // Try to load previously unloaded files with the new loader
        self.retry_unloaded_files().await;
    }

    /// Retry loading files that couldn't be loaded previously
    async fn retry_unloaded_files(&self) {
        let files_to_retry: Vec<PathBuf> =
            { self.unloaded_files.read().await.iter().cloned().collect() };
        let mut retry_tasks = Vec::new();

        for path in files_to_retry {
            if let Ok(task) = self.start_loading_plugin(&path).await {
                retry_tasks.push(task);
            }
        }

        // Wait for all retry tasks to complete
        join_all(retry_tasks).await;
    }

    /// Set server reference for plugin context
    pub async fn set_server(&self, server: Arc<Server>) {
        let mut srv = self.server.write().await;
        srv.replace(server);
    }

    /// Set self reference for creating contexts
    pub async fn set_self_ref(&self, self_ref: Arc<Self>) {
        let mut sref = self.self_ref.write().await;
        sref.replace(self_ref);
    }

    /// Get a clone of the loaders for context use
    #[must_use]
    pub async fn get_loaders(&self) -> Vec<Arc<dyn PluginLoader>> {
        self.loaders.read().await.clone()
    }

    /// Load all plugins from the plugin directory
    pub async fn load_plugins(&self) -> Result<(), ManagerError> {
        const PLUGIN_DIR: &str = "./plugins";
        let path = Path::new(PLUGIN_DIR);

        if !path.exists() {
            std::fs::create_dir(path)?;
            return Ok(());
        }

        let mut load_tasks = Vec::new();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            // Start loading plugin concurrently
            if let Ok(task) = self.start_loading_plugin(&path).await {
                load_tasks.push(task);
            }
        }

        // Wait for all plugins to complete loading (but don't block on individual plugin initialization)
        join_all(load_tasks).await;

        Ok(())
    }

    /// Start loading a plugin asynchronously
    async fn start_loading_plugin(
        &self,
        path: &Path,
    ) -> Result<tokio::task::JoinHandle<()>, ManagerError> {
        let loaders = self.loaders.read().await;
        for loader in loaders.iter() {
            if loader.can_load(path) {
                let server = self.server.read().await;
                let self_ref = self.self_ref.read().await;

                let (mut instance, metadata, loader_data) = loader.load(path).await?;

                // Mark plugin as loading
                self.plugin_states
                    .write()
                    .await
                    .insert(metadata.name.to_string(), PluginState::Loading);

                let self_ref = self_ref
                    .as_ref()
                    .ok_or(ManagerError::ServerNotInitialized)?;

                let context = Arc::new(Context::new(
                    metadata.clone(),
                    Arc::clone(server.as_ref().ok_or(ManagerError::ServerNotInitialized)?),
                    Arc::clone(&self.handlers),
                    Arc::clone(self_ref),
                    Arc::clone(&PERMISSION_MANAGER),
                ));

                // Create the plugin structure first
                let plugin = LoadedPlugin {
                    metadata: metadata.clone(),
                    instance: None, // Will be set after successful initialization
                    loader: loader.clone(),
                    loader_data: Some(loader_data),
                    is_active: false, // Will be set to true after successful initialization
                    context: context.clone(),
                };

                let plugin_index = {
                    let mut plugins = self.plugins.write().await;
                    plugins.push(plugin);
                    plugins.len() - 1
                };

                // Remove from unloaded files if it was there
                self.unloaded_files.write().await.remove(path);

                // Spawn async task for plugin initialization
                let self_ref_clone = Arc::clone(self_ref);
                let state_notify = Arc::clone(&self.state_notify);
                let plugin_name = metadata.name.to_string();
                let loader_clone = loader.clone();

                let task = tokio::spawn(async move {
                    // Initialize the plugin
                    match instance.on_load(context.clone()).await {
                        Ok(()) => {
                            // Update plugin state to loaded
                            {
                                let mut plugins = self_ref_clone.plugins.write().await;
                                if let Some(plugin) = plugins.get_mut(plugin_index) {
                                    plugin.instance = Some(instance);
                                    plugin.is_active = true;
                                }
                            }
                            self_ref_clone
                                .plugin_states
                                .write()
                                .await
                                .insert(plugin_name.clone(), PluginState::Loaded);
                            state_notify.notify_waiters();

                            log::info!("Loaded {} ({})", metadata.name, metadata.version);
                        }
                        Err(e) => {
                            // Handle initialization failure
                            let error_msg = format!("Initialization failed: {e}");
                            let _ = instance.on_unload(context).await;

                            // Get the loader data before removing the plugin
                            let loader_data: Option<Box<dyn Any + Send + Sync>> = {
                                let mut plugins = self_ref_clone.plugins.write().await;
                                if let Some(plugin) = plugins.get_mut(plugin_index) {
                                    plugin.loader_data.take()
                                } else {
                                    None
                                }
                            };

                            // Try to unload the plugin data
                            if let Some(data) = loader_data {
                                tokio::spawn(async move {
                                    loader_clone.unload(data).await.ok();
                                });
                            }

                            {
                                let mut plugins = self_ref_clone.plugins.write().await;
                                if plugin_index < plugins.len() {
                                    plugins.remove(plugin_index);
                                }
                            };
                            self_ref_clone.plugin_states.write().await.insert(
                                plugin_name.clone(),
                                PluginState::Failed(error_msg.clone()),
                            );
                            state_notify.notify_waiters();

                            log::error!("Failed to initialize plugin {plugin_name}: {error_msg}",);
                        }
                    }
                });

                return Ok(task);
            }
        }

        // No loader could handle this file, track it for future attempts
        self.unloaded_files.write().await.insert(path.to_path_buf());

        Err(ManagerError::PluginNotFound(
            path.to_string_lossy().to_string(),
        ))
    }

    /// Attempt to load a single plugin file
    pub async fn try_load_plugin(&self, path: &Path) -> Result<(), ManagerError> {
        self.start_loading_plugin(path).await?.await.map_err(|e| {
            ManagerError::LoaderError(LoaderError::InitializationFailed(format!(
                "Task join error: {e}"
            )))
        })
    }

    /// Wait for a plugin to finish loading
    pub async fn wait_for_plugin(&self, plugin_name: &str) -> Result<(), ManagerError> {
        loop {
            let state = self.plugin_states.read().await.get(plugin_name).cloned();
            if let Some(state) = state {
                match state {
                    PluginState::Loaded => return Ok(()),
                    PluginState::Failed(error) => {
                        return Err(ManagerError::LoaderError(
                            LoaderError::InitializationFailed(error.clone()),
                        ));
                    }
                    PluginState::Loading => {
                        // Wait for state change notification
                        self.state_notify.notified().await;
                        continue;
                    }
                }
            }
            return Err(ManagerError::PluginNotFound(plugin_name.to_string()));
        }
    }

    /// Get the current state of a plugin
    pub async fn get_plugin_state(&self, plugin_name: &str) -> Option<PluginState> {
        self.plugin_states.read().await.get(plugin_name).cloned()
    }

    /// Checks if plugin active
    #[must_use]
    pub async fn is_plugin_active(&self, name: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .any(|p| p.metadata.name == name && p.is_active && p.instance.is_some())
    }

    /// Get list of active plugins
    #[must_use]
    pub async fn active_plugins(&self) -> Vec<PluginMetadata<'static>> {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .filter(|p| p.is_active && p.instance.is_some())
            .map(|p| p.metadata.clone())
            .collect()
    }

    /// Checks if plugin loaded
    #[must_use]
    pub async fn is_plugin_loaded(&self, name: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.iter().any(|p| p.metadata.name == name)
    }

    /// Get list of loaded plugins
    #[must_use]
    pub async fn loaded_plugins(&self) -> Vec<PluginMetadata<'static>> {
        let plugins = self.plugins.read().await;
        plugins.iter().map(|p| p.metadata.clone()).collect()
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&self, name: &str) -> Result<(), ManagerError> {
        let index = {
            let plugins = self.plugins.read().await;
            plugins
                .iter()
                .position(|p| p.metadata.name == name)
                .ok_or_else(|| ManagerError::PluginNotFound(name.to_string()))?
        };

        let mut plugin = {
            let mut plugins = self.plugins.write().await;
            plugins.remove(index)
        };

        if let Some(mut instance) = plugin.instance.take() {
            instance.on_unload(plugin.context.clone()).await.ok();
        }

        if plugin.loader.can_unload() {
            if let Some(data) = plugin.loader_data {
                plugin.loader.unload(data).await?;
            }
        } else {
            plugin.is_active = false;
            self.plugins.write().await.push(plugin);
        }

        // Remove from plugin states
        self.plugin_states.write().await.remove(name);

        Ok(())
    }

    /// Get all plugins that are currently loading
    pub async fn get_loading_plugins(&self) -> Vec<String> {
        let plugin_states = self.plugin_states.read().await;
        plugin_states
            .iter()
            .filter(|(_, state)| matches!(state, PluginState::Loading))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get all plugins that failed to load
    pub async fn get_failed_plugins(&self) -> Vec<(String, String)> {
        let plugin_states = self.plugin_states.read().await;
        plugin_states
            .iter()
            .filter_map(|(name, state)| {
                if let PluginState::Failed(error) = state {
                    Some((name.clone(), error.clone()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if all plugins have finished loading (either succeeded or failed)
    pub async fn all_plugins_loaded(&self) -> bool {
        let plugin_states = self.plugin_states.read().await;
        !plugin_states
            .values()
            .any(|state| matches!(state, PluginState::Loading))
    }

    /// Wait for all plugins to finish loading
    pub async fn wait_for_all_plugins(&self) {
        while !self.all_plugins_loaded().await {
            self.state_notify.notified().await;
        }
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
        if let Some(server) = self.server.read().await.as_ref() {
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
