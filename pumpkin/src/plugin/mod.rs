use loader::{LoaderError, PluginLoader, native::NativePluginLoader};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    path::PathBuf,
    pin::Pin,
    sync::{Arc, atomic::AtomicBool},
};
use thiserror::Error;
use tokio::{
    sync::{Notify, RwLock},
    task::JoinHandle,
};

pub mod api;
pub mod cache;
pub mod loader;
/// Constants for plugin permissions.
///
/// Plugins can request these permissions in their metadata to access specific
/// host features.
pub mod permissions;

mod events;
mod lifecycle;
mod loading;

use crate::{plugin::loader::wasm::WasmPluginLoader, server::Server};
pub use api::*;
pub use events::{DynEventHandler, EventHandler};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Bump this whenever the public plugin API or any event layout changes in a way
/// that makes old binary plugins incompatible.
pub const PLUGIN_API_VERSION: u32 = 2;

const PLUGIN_DIR: &str = "./plugins";

/// A struct representing a typed event handler.
///
/// This struct holds a reference to an event handler, its priority, and whether it is blocking.
struct TypedEventHandler<E, H>
where
    E: Payload + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    handler: Arc<H>,
    priority: EventPriority,
    blocking: bool,
    _phantom: std::marker::PhantomData<E>,
}

impl<E, H> DynEventHandler for TypedEventHandler<E, H>
where
    E: Payload + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    /// Asynchronously handles a blocking dynamic event.
    fn handle_blocking_dyn<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut (dyn Payload + Send + Sync),
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if let Some(typed_event) = <dyn Payload>::downcast_mut(event) {
                // The handler.handle_blocking call now returns a Future, which we await.
                self.handler.handle_blocking(server, typed_event).await;
            }
        })
    }

    /// Asynchronously handles a dynamic event.
    fn handle_dyn<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a (dyn Payload + Send + Sync),
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if let Some(typed_event) = <dyn Payload>::downcast_ref(event) {
                // The handler.handle call now returns a Future, which we await.
                self.handler.handle(server, typed_event).await;
            }
        })
    }

    /// Checks if the handler is blocking.
    fn is_blocking(&self) -> bool {
        self.blocking
    }

    /// Retrieves the priority of the handler.
    fn get_priority(&self) -> &EventPriority {
        &self.priority
    }
}

/// A type alias for a map of event handlers, where the key is a static string
/// and the value is a vector of dynamic event handlers.
type HandlerMap = HashMap<&'static str, Vec<Box<dyn DynEventHandler>>>;

/// Plugin loading state
#[derive(Debug, Clone, PartialEq, Eq)]
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
    self_ref: RwLock<Option<Arc<Self>>>,
    services: Arc<RwLock<HashMap<String, Arc<dyn Payload>>>>,
    // Plugin state tracking
    plugin_states: RwLock<HashMap<String, PluginState>>,
    // Notification for plugin state changes
    state_notify: Arc<Notify>,
    // Background task for hot reloading
    hot_reload_task: RwLock<Option<JoinHandle<()>>>,
    hot_reload_enabled: AtomicBool,
}

/// Represents a successfully loaded plugin
///
/// OS specific issues
/// - Windows: Plugin cannot be unloaded, it can be only active or not
struct LoadedPlugin {
    metadata: PluginMetadata,
    instance: Option<Box<dyn Plugin>>,
    loader: Arc<dyn PluginLoader>,
    loader_data: Option<Box<dyn Any + Send + Sync>>,
    is_active: bool,
    context: Arc<Context>,
    path: PathBuf,
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

    #[error("Dependency error: {0}")]
    DependencyError(String),
}

impl Default for PluginManager {
    fn default() -> Self {
        Self {
            plugins: RwLock::new(Vec::new()),
            loaders: RwLock::new(vec![
                Arc::new(NativePluginLoader),
                Arc::new(WasmPluginLoader),
            ]),
            server: RwLock::new(None),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            unloaded_files: RwLock::new(HashSet::new()),
            self_ref: RwLock::new(None),
            services: Arc::new(RwLock::new(HashMap::new())),
            plugin_states: RwLock::new(HashMap::new()),
            state_notify: Arc::new(Notify::new()),
            hot_reload_task: RwLock::new(None),
            hot_reload_enabled: AtomicBool::new(false),
        }
    }
}

impl PluginManager {
    /// Create a new plugin manager with default loaders
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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
}
