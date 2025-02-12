pub mod api;

pub use api::*;
use async_trait::async_trait;
use std::{collections::HashMap, fs, path::Path, sync::Arc};
use tokio::sync::RwLock;

use crate::server::Server;
use thiserror::Error;

type PluginData = (
    PluginMetadata<'static>,
    Box<dyn Plugin>,
    libloading::Library,
    bool,
);

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
pub type HandlerMap = HashMap<&'static str, Vec<Box<dyn DynEventHandler>>>;

/// A struct for managing plugins.
pub struct PluginManager {
    plugins: Vec<PluginData>,
    server: Option<Arc<Server>>,
    handlers: Arc<RwLock<HandlerMap>>,
}

impl Default for PluginManager {
    /// Creates a new instance of `PluginManager` with default values.
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    /// Creates a new instance of `PluginManager`.
    ///
    /// # Returns
    /// A new instance of `PluginManager`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            server: None,
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sets the server reference for the plugin manager.
    ///
    /// # Arguments
    /// - `server`: An `Arc` reference to the server to set.
    pub fn set_server(&mut self, server: Arc<Server>) {
        self.server = Some(server);
    }

    /// Asynchronously loads plugins from the specified plugin directory.
    ///
    /// # Returns
    /// A result indicating success or failure. If it fails, it returns a `PluginsLoadError`.
    pub async fn load_plugins(&mut self) -> Result<(), PluginsLoadError> {
        const PLUGIN_DIR: &str = "./plugins";

        if !Path::new(PLUGIN_DIR).exists() {
            fs::create_dir(PLUGIN_DIR).map_err(|_| PluginsLoadError::CreatePluginDir)?;
            // If the directory was just created, it should be empty, so we return.
            return Ok(());
        }

        let dir_entries = fs::read_dir(PLUGIN_DIR).map_err(|_| PluginsLoadError::ReadPluginDir)?;

        for entry in dir_entries {
            let entry = entry.unwrap();
            if !entry.file_type().unwrap().is_file() {
                continue;
            }
            let name = entry.file_name().into_string().unwrap();
            if let Err(err) = self.try_load_plugin(&entry.path()).await {
                log::error!("Plugin {}: {}", name, err.to_string());
            }
        }

        Ok(())
    }

    /// Tries to load a plugin from the specified path.
    ///
    /// # Arguments
    /// - `path`: The path to the plugin to load.
    ///
    /// # Returns
    /// A result indicating success or failure. If it fails, it returns a `PluginLoadError`.
    async fn try_load_plugin(&mut self, path: &Path) -> Result<(), PluginLoadError> {
        let library = unsafe {
            libloading::Library::new(path)
                .map_err(|e| PluginLoadError::LoadLibrary(e.to_string()))?
        };

        let plugin_fn = unsafe {
            library
                .get::<fn() -> Box<dyn Plugin>>(b"plugin")
                .map_err(|_| PluginLoadError::GetPluginMain)?
        };
        let metadata: &PluginMetadata = unsafe {
            &**library
                .get::<*const PluginMetadata>(b"METADATA")
                .map_err(|_| PluginLoadError::GetPluginMeta)?
        };

        // Create a context for the plugin.
        let context = Context::new(
            metadata.clone(),
            self.server.clone().expect("Server not set"),
            self.handlers.clone(),
        );
        let mut plugin_box = plugin_fn();
        let res = plugin_box.on_load(&context).await;
        let mut loaded = true;
        if let Err(e) = res {
            log::error!("Error loading plugin: {}", e);
            loaded = false;
        }

        self.plugins
            .push((metadata.clone(), plugin_box, library, loaded));
        Ok(())
    }

    /// Checks if a plugin is loaded by its name.
    ///
    /// # Arguments
    /// - `name`: The name of the plugin to check.
    ///
    /// # Returns
    /// A boolean indicating whether the plugin is loaded.
    #[must_use]
    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.plugins
            .iter()
            .any(|(metadata, _, _, loaded)| metadata.name == name && *loaded)
    }

    /// Asynchronously loads a plugin by its name.
    ///
    /// # Arguments
    /// - `name`: The name of the plugin to load.
    ///
    /// # Returns
    /// A result indicating success or failure. If it fails, it returns an error message.
    pub async fn load_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|(metadata, _, _, _)| metadata.name == name);

        if let Some((metadata, plugin, _, loaded)) = plugin {
            if *loaded {
                return Err(format!("Plugin {name} is already loaded"));
            }

            let context = Context::new(
                metadata.clone(),
                self.server.clone().expect("Server not set"),
                self.handlers.clone(),
            );
            let res = plugin.on_load(&context).await;
            res?;
            *loaded = true;
            Ok(())
        } else {
            Err(format!("Plugin {name} not found"))
        }
    }

    /// Asynchronously unloads a plugin by its name.
    ///
    /// # Arguments
    /// - `name`: The name of the plugin to unload.
    ///
    /// # Returns
    /// A result indicating success or failure. If it fails, it returns an error message.
    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|(metadata, _, _, _)| metadata.name == name);

        if let Some((metadata, plugin, _, loaded)) = plugin {
            let context = Context::new(
                metadata.clone(),
                self.server.clone().expect("Server not set"),
                self.handlers.clone(),
            );
            let res = plugin.on_unload(&context).await;
            res?;
            *loaded = false;
            Ok(())
        } else {
            Err(format!("Plugin {name} not found"))
        }
    }

    /// Lists all plugins along with their loaded status.
    ///
    /// # Returns
    /// A vector of tuples containing references to the plugin metadata and a boolean indicating
    /// whether each plugin is loaded.
    #[must_use]
    pub fn list_plugins(&self) -> Vec<(&PluginMetadata, &bool)> {
        self.plugins
            .iter()
            .map(|(metadata, _, _, loaded)| (metadata, loaded))
            .collect()
    }

    /// Asynchronously registers an event handler for a specific event type.
    ///
    /// # Type Parameters
    /// - `E`: The event type that the handler will respond to.
    /// - `H`: The type of the event handler.
    ///
    /// # Arguments
    /// - `handler`: A reference to the event handler.
    /// - `priority`: The priority of the event handler.
    /// - `blocking`: A boolean indicating whether the handler is blocking.
    ///
    /// # Constraints
    /// The handler must implement the `EventHandler<E>` trait.
    pub async fn register<E: Event + 'static, H>(
        &self,
        handler: Arc<H>,
        priority: EventPriority,
        blocking: bool,
    ) where
        H: EventHandler<E> + 'static,
    {
        let mut handlers = self.handlers.write().await;

        let handlers_vec = handlers
            .entry(E::get_name_static())
            .or_insert_with(Vec::new);

        let typed_handler = TypedEventHandler {
            handler,
            priority,
            blocking,
            _phantom: std::marker::PhantomData,
        };

        handlers_vec.push(Box::new(typed_handler));
    }

    /// Asynchronously fires an event, invoking all registered handlers for that event type.
    ///
    /// # Type Parameters
    /// - `E`: The event type to fire.
    ///
    /// # Arguments
    /// - `event`: The event to fire.
    ///
    /// # Returns
    /// The event after all handlers have processed it.
    pub async fn fire<E: Event + Send + Sync + 'static>(&self, mut event: E) -> E {
        // Take a snapshot of handlers to avoid lifetime issues
        let handlers = self.handlers.read().await;

        log::trace!("Firing event: {}", E::get_name_static());

        if let Some(handlers_vec) = handlers.get(&E::get_name_static()) {
            log::trace!(
                "Found {} handlers for event: {}",
                handlers_vec.len(),
                E::get_name_static()
            );

            let (blocking_handlers, non_blocking_handlers): (Vec<_>, Vec<_>) = handlers_vec
                .iter()
                .partition(|handler| handler.is_blocking());

            // Handle blocking handlers first
            for handler in blocking_handlers {
                handler.handle_blocking_dyn(&mut event).await;
            }

            // TODO: Run non-blocking handlers in parallel
            for handler in non_blocking_handlers {
                handler.handle_dyn(&event).await;
            }
        }

        event
    }
}

/// Error when failed to load the entire Plugin directory
#[derive(Error, Debug)]
pub enum PluginsLoadError {
    #[error("Failed to Create new Plugins directory")]
    CreatePluginDir,
    #[error("Failed to Read Plugins directory")]
    ReadPluginDir,
    #[error("Failed to load Plugin {0}")]
    LoadPlugin(String, PluginLoadError),
}

/// Error when failed to load a single Plugin
#[derive(Error, Debug)]
pub enum PluginLoadError {
    #[error("Failed to load Library: {0}")]
    LoadLibrary(String),
    #[error("Failed to load Plugin entry function")]
    GetPluginMain,
    #[error("Failed to load Plugin Metadata")]
    GetPluginMeta,
}
