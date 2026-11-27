use arc_swap::ArcSwap;
use futures::future::join_all;
use loader::{LoaderError, PluginLoader, native::NativePluginLoader};
use notify::{EventKind, RecursiveMode, Watcher, event::ModifyKind};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    pin::{Pin, pin},
    sync::{Arc, RwLock as SyncRwLock, atomic::AtomicBool},
    thread::ThreadId,
    time::Duration,
};
use thiserror::Error;
use tokio::{
    sync::{Notify, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock},
    task::JoinHandle,
};
use tracing::{debug, error, info, warn};

pub mod api;
pub mod cache;
pub mod loader;
/// Constants for plugin permissions.
///
/// Plugins can request these permissions in their metadata to access specific
/// host features.
pub mod permissions;

use crate::{
    LOGGER_IMPL,
    net::{bedrock::BedrockVersions, java::JavaVersions},
    server::Server,
};
pub use api::*;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Bump this whenever the public plugin API or any event layout changes in a way
/// that makes old binary plugins incompatible.
pub const PLUGIN_API_VERSION: u32 = 8;

const PLUGIN_DIR: &str = "./plugins";

/// Access to an event whose handler can continue running after its caller stops waiting.
pub struct QueuedEvent(QueuedEventInner);

enum QueuedEventInner {
    ReadOnly(OwnedRwLockReadGuard<dyn EventSlot>),
    ReadWrite(OwnedRwLockWriteGuard<dyn EventSlot>),
}

impl QueuedEvent {
    /// Borrows a live event of the requested type for one synchronous host accessor.
    ///
    /// Returns `None` if the type differs or dispatch has reclaimed the payload.
    #[must_use]
    pub fn read<E: Payload + 'static>(&self) -> Option<&E> {
        match &self.0 {
            QueuedEventInner::ReadOnly(guard) => guard.payload()?.downcast_ref(),
            QueuedEventInner::ReadWrite(guard) => guard.payload()?.downcast_ref(),
        }
    }

    /// Borrows a live event for an edit by a blocking handler.
    ///
    /// Returns `None` for notifications, a type mismatch, or a reclaimed payload.
    #[must_use]
    pub fn write<E: Payload + 'static>(&mut self) -> Option<&mut E> {
        match &mut self.0 {
            QueuedEventInner::ReadOnly(_) => None,
            QueuedEventInner::ReadWrite(guard) => guard.payload_mut()?.downcast_mut(),
        }
    }
}

// The erased lock and typed lock both refer to the same allocation.
trait EventSlot: Send + Sync {
    fn payload(&self) -> Option<&dyn Payload>;
    fn payload_mut(&mut self) -> Option<&mut dyn Payload>;
}

impl<E: Payload + 'static> EventSlot for Option<E> {
    fn payload(&self) -> Option<&dyn Payload> {
        self.as_ref().map(|event| event as &dyn Payload)
    }

    fn payload_mut(&mut self) -> Option<&mut dyn Payload> {
        self.as_mut().map(|event| event as &mut dyn Payload)
    }
}

/// A trait for handling events dynamically.
///
/// This trait allows for handling events of any type that implements the `Event` trait.
pub trait DynEventHandler: Send + Sync {
    /// Asynchronously handles a dynamic event.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    fn handle_dyn<'a>(&'a self, server: &'a Arc<Server>, event: QueuedEvent) -> BoxFuture<'a, ()>;

    /// Checks if the event handler is blocking.
    ///
    /// # Returns
    /// A boolean indicating whether the handler is blocking.
    fn is_blocking(&self) -> bool;

    /// Retrieves the priority of the event handler.
    ///
    /// # Returns
    /// The priority of the event handler.
    fn get_priority(&self) -> &EventPriority;

    /// Returns the plugin that registered this handler, when applicable.
    fn source(&self) -> Option<&str> {
        None
    }
}

/// A trait for handling specific events.
///
/// These handlers are the non-blocking variant (ran concurrently), and
/// are given read-only access to the event (conforming to the `Event` trait).
pub trait EventRoHandler<E: Payload>: Send + Sync {
    /// Asynchronously handles an event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    fn handle<'a>(&'a self, _server: &'a Arc<Server>, _event: &'a E) -> BoxFuture<'a, ()>;
}

/// A trait for handling specific events.
///
/// These handlers are the blocking variant (ran sequentially), and are allowed
/// to both read AND write the event (conforming to the `Event` trait).
pub trait EventRwHandler<E: Payload>: Send + Sync {
    /// Asynchronously handles a blocking event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    fn handle<'a>(&'a self, _server: &'a Arc<Server>, _event: &'a mut E) -> BoxFuture<'a, ()>;
}

/// A struct representing a typed read-only/non-blocking event handler.
///
/// This struct holds a reference to an event handler, and its priority.
pub struct TypedEventRoHandler<E, H>
where
    E: Payload + 'static,
    H: EventRoHandler<E>,
{
    pub handler: Arc<H>,
    pub priority: EventPriority,
    pub source: Option<String>,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E, H> DynEventHandler for TypedEventRoHandler<E, H>
where
    E: Payload + 'static,
    H: EventRoHandler<E>,
{
    /// Asynchronously handles a dynamic event.
    fn handle_dyn<'a>(&'a self, server: &'a Arc<Server>, event: QueuedEvent) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let Some(payload) = event.read::<E>() else {
                tracing::error!(
                    "A handler for `{}` failed. Unable to read access to event. Skipped.",
                    E::get_name_static()
                );
                return;
            };

            self.handler.handle(server, payload).await;
        })
    }

    /// Checks if the handler is blocking.
    fn is_blocking(&self) -> bool {
        false
    }

    /// Retrieves the priority of the handler.
    fn get_priority(&self) -> &EventPriority {
        &self.priority
    }

    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

/// A struct representing a typed read-write/blocking event handler.
///
/// This struct holds a reference to an event handler, and its priority.
pub struct TypedEventRwHandler<E, H>
where
    E: Payload + 'static,
    H: EventRwHandler<E>,
{
    pub handler: Arc<H>,
    pub priority: EventPriority,
    pub source: Option<String>,
    pub _phantom: std::marker::PhantomData<E>,
}

impl<E, H> DynEventHandler for TypedEventRwHandler<E, H>
where
    E: Payload + 'static,
    H: EventRwHandler<E>,
{
    /// Asynchronously handles a dynamic event.
    fn handle_dyn<'a>(
        &'a self,
        server: &'a Arc<Server>,
        mut event: QueuedEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let Some(payload) = event.write::<E>() else {
                tracing::error!(
                    "A handler for `{}` failed. Unable to get write access to event. Skipped.",
                    E::get_name_static()
                );
                return;
            };

            self.handler.handle(server, payload).await;
        })
    }

    /// Checks if the handler is blocking.
    fn is_blocking(&self) -> bool {
        true
    }

    /// Retrieves the priority of the handler.
    fn get_priority(&self) -> &EventPriority {
        &self.priority
    }

    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

/// A type alias for a map of event handlers, where the key is a static string
/// and the value is a vector of dynamic event handlers.
pub type HandlerMap = HashMap<&'static str, Vec<Arc<dyn DynEventHandler>>>;

/// Plugin loading state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Loading,
    Loaded,
    Failed(String),
}

/// Core plugin management system
#[repr(C)]
pub struct PluginManager {
    plugins: SyncRwLock<Vec<LoadedPlugin>>,
    loaders: RwLock<Vec<Arc<dyn PluginLoader>>>,
    pub handlers: Arc<ArcSwap<HandlerMap>>,
    pub java_versions: Arc<JavaVersions>,
    pub bedrock_versions: Arc<BedrockVersions>,
    unloaded_files: RwLock<HashSet<PathBuf>>,
    services: Arc<RwLock<HashMap<String, Arc<dyn Payload>>>>,
    // Plugin state tracking
    plugin_states: RwLock<HashMap<String, PluginState>>,
    // Notification for plugin state changes
    state_notify: Arc<Notify>,
    // Background task for hot reloading
    hot_reload_task: RwLock<Option<JoinHandle<()>>>,
    hot_reload_enabled: AtomicBool,
    // Thread ID of the thread that created the plugin manager.
    // Permission prompts use rustyline, which is only safe on this thread.
    main_thread_id: ThreadId,
}

/// Represents a successfully loaded plugin
///
/// OS specific issues
/// - Windows: Plugin cannot be unloaded, it can be only active or not
struct LoadedPlugin {
    metadata: PluginMetadata,
    instance: Option<Arc<dyn Plugin>>,
    loader: Arc<dyn PluginLoader>,
    loader_data: Option<Box<dyn Any + Send + Sync>>,
    is_active: bool,
    context: Arc<Context>,
    path: PathBuf,
}

/// Error types for plugin management
#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Plugin not found: {0}")]
    PluginNotFound(String),
    #[error("Loader error: {0}")]
    LoaderError(#[from] LoaderError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Dependency error: {0}")]
    DependencyError(String),
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl PluginManager {
    /// Create a new plugin manager with the native loader and the given extra loaders
    #[must_use]
    pub fn new(extra_loaders: Vec<Arc<dyn PluginLoader>>) -> Self {
        let mut loaders: Vec<Arc<dyn PluginLoader>> = vec![Arc::new(NativePluginLoader)];
        loaders.extend(extra_loaders);
        Self {
            plugins: SyncRwLock::new(Vec::new()),
            loaders: RwLock::new(loaders),
            handlers: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            java_versions: Arc::new(JavaVersions::new()),
            bedrock_versions: Arc::new(BedrockVersions::new()),
            unloaded_files: RwLock::new(HashSet::new()),
            services: Arc::new(RwLock::new(HashMap::new())),
            plugin_states: RwLock::new(HashMap::new()),
            state_notify: Arc::new(Notify::new()),
            hot_reload_task: RwLock::new(None),
            hot_reload_enabled: AtomicBool::new(false),
            main_thread_id: std::thread::current().id(),
        }
    }

    /// Unload all loaded plugins
    pub async fn unload_all_plugins(&self) -> Result<(), ManagerError> {
        let plugin_names: Vec<String> = {
            let plugins = self
                .plugins
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plugins
                .iter()
                .filter(|p| p.is_active)
                .map(|p| p.metadata.name.clone())
                .collect()
        };

        for name in plugin_names {
            if let Err(e) = self.unload_plugin(&name).await {
                error!("Failed to unload plugin {name}: {e}");
            }
        }

        Ok(())
    }

    /// Add a new plugin loader implementation
    pub async fn add_loader(self: &Arc<Self>, server: &Arc<Server>, loader: Arc<dyn PluginLoader>) {
        self.loaders.write().await.push(loader);

        // Try to load previously unloaded files with the new loader
        self.retry_unloaded_files(server).await;
    }

    /// Start watching the plugins directory for changes
    pub async fn start_watcher(self: &Arc<Self>, server: &Arc<Server>) -> Result<(), ManagerError> {
        if self.hot_reload_task.read().await.is_some() {
            return Ok(());
        }

        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })
        .map_err(|e| ManagerError::IoError(std::io::Error::other(e)))?;

        let plugin_dir = Path::new(PLUGIN_DIR);
        if !plugin_dir.exists() {
            std::fs::create_dir_all(plugin_dir)?;
        }

        watcher
            .watch(plugin_dir, RecursiveMode::NonRecursive)
            .map_err(|e| ManagerError::IoError(std::io::Error::other(e)))?;

        let manager = self.clone();
        let server_clone = Arc::clone(server);
        let task = server.spawn_task(async move {
            // Keep watcher alive by moving it into the task
            let _watcher = watcher;

            while let Some(event) = rx.recv().await {
                if !manager
                    .hot_reload_enabled
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    continue;
                }

                match event.kind {
                    EventKind::Modify(ModifyKind::Data(_)) | EventKind::Create(_) => {
                        for path in event.paths {
                            if path.extension().is_some_and(|ext| ext == "wasm") {
                                debug!("Detected change in plugin: {:?}", path);
                                // Give it a small delay to ensure file is completely written
                                tokio::time::sleep(Duration::from_millis(100)).await;

                                // We need to find if this plugin is already loaded to unload it first
                                let plugin_name = {
                                    let plugins = manager
                                        .plugins
                                        .read()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                                    plugins
                                        .iter()
                                        .find(|p| p.path == path)
                                        .map(|p| p.metadata.name.clone())
                                };

                                if let Some(name) = plugin_name {
                                    info!("Hot-reloading plugin: {}", name);
                                    let _ = manager.unload_plugin(&name).await;
                                }

                                // For now, we just try to load it. If it's already loaded,
                                // the loader might handle it or we might get a duplicate.
                                // Most WASM loaders will just create a new instance.
                                if let Err(e) =
                                    manager.start_loading_plugin(&server_clone, &path).await
                                {
                                    error!("Failed to hot-reload plugin {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        *self.hot_reload_task.write().await = Some(task);
        self.set_hot_reload_enabled(true);
        Ok(())
    }

    /// Stop watching the plugins directory for changes
    pub async fn stop_watcher(&self) {
        let mut task_lock = self.hot_reload_task.write().await;
        if let Some(handle) = task_lock.take() {
            handle.abort();
        }
        self.set_hot_reload_enabled(false);
    }

    pub fn set_hot_reload_enabled(&self, enabled: bool) {
        self.hot_reload_enabled
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn is_hot_reload_enabled(&self) -> bool {
        self.hot_reload_enabled
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Retry loading files that couldn't be loaded previously
    async fn retry_unloaded_files(self: &Arc<Self>, server: &Arc<Server>) {
        let files_to_retry: Vec<PathBuf> =
            { self.unloaded_files.read().await.iter().cloned().collect() };
        let mut retry_tasks = Vec::new();

        for path in files_to_retry {
            if let Ok(task) = self.start_loading_plugin(server, &path).await {
                retry_tasks.push(task);
            }
        }

        // Wait for all retry tasks to complete
        join_all(retry_tasks).await;
    }

    /// Lets every loader run its per-tick work.
    pub fn tick_loaders(&self, server: &Arc<Server>) {
        if let Ok(loaders) = self.loaders.try_read() {
            for loader in loaders.iter() {
                loader.on_tick(server);
            }
        }
    }

    /// Get a clone of the loaders for context use
    #[must_use]
    pub async fn get_loaders(&self) -> Vec<Arc<dyn PluginLoader>> {
        self.loaders.read().await.clone()
    }

    /// Helper for topological sort of plugins based on dependencies
    fn topological_sort(plugins: &[(String, Vec<String>)]) -> Result<Vec<String>, String> {
        fn visit(
            name: &str,
            deps_map: &HashMap<String, &Vec<String>>,
            plugin_names: &HashSet<String>,
            visited: &mut HashSet<String>,
            current_path: &mut HashSet<String>,
            sorted: &mut Vec<String>,
        ) -> Result<(), String> {
            if current_path.contains(name) {
                return Err(format!(
                    "Circular dependency detected involving plugin: {name}"
                ));
            }
            if !visited.contains(name) {
                current_path.insert(name.to_string());
                if let Some(deps) = deps_map.get(name) {
                    for dep in *deps {
                        if !plugin_names.contains(dep) {
                            return Err(format!("Plugin {name} depends on missing plugin: {dep}"));
                        }
                        visit(dep, deps_map, plugin_names, visited, current_path, sorted)?;
                    }
                }
                current_path.remove(name);
                visited.insert(name.to_string());
                sorted.push(name.to_string());
            }
            Ok(())
        }
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut current_path = HashSet::new();
        let plugin_names: HashSet<String> = plugins.iter().map(|(n, _)| n.clone()).collect();
        let deps_map: HashMap<String, &Vec<String>> =
            plugins.iter().map(|(n, d)| (n.clone(), d)).collect();

        for (name, _) in plugins {
            visit(
                name,
                &deps_map,
                &plugin_names,
                &mut visited,
                &mut current_path,
                &mut sorted,
            )?;
        }

        Ok(sorted)
    }

    /// Ask the server owner if they allow the permissions requested by a plugin
    ///
    /// This must only be called from the main thread, because the underlying
    /// `rustyline` console handle is neither `Send` nor `Sync`. Calling it from a
    /// different thread (e.g. the hot-reload watcher task) will panic.
    ///
    /// Returns `(allowed, wait_time, cacheable)`. Non-main-thread denials are not
    /// cacheable so the user gets prompted again on the next cold load.
    #[expect(clippy::print_stdout)]
    fn ask_permission_confirmation(
        &self,
        metadata: &PluginMetadata,
    ) -> (bool, std::time::Duration, bool) {
        use colored::Colorize;

        if metadata.permissions.is_empty() {
            return (true, std::time::Duration::ZERO, true);
        }

        if std::thread::current().id() != self.main_thread_id {
            warn!(
                "Plugin \"{}\" ({}) requests permissions from a non-main thread. \
                 Permission prompts are only supported on the main thread. \
                 Denying permissions. To load this plugin interactively, restart the server.",
                metadata.name, metadata.version
            );
            return (false, Duration::ZERO, false);
        }

        let start_time = std::time::Instant::now();

        println!(
            "\n{} \"{}\" ({}) requests the following permissions:",
            "Plugin".bold(),
            metadata.name.cyan(),
            metadata.version.green()
        );
        for permission in &metadata.permissions {
            println!(
                "  - {}: {}",
                permission.yellow().bold(),
                permissions::get_permission_description(permission)
                    .unwrap_or("<unknown permission>")
                    .italic()
            );
        }

        let prompt = format!(
            "\n{} [y/N]: ",
            "Do you want to allow these permissions and load the plugin?".bold()
        );

        let mut rl_taken = if let Some(logger_option) = crate::LOGGER_IMPL.get()
            && let Some((wrapper, _, _)) = logger_option
            && let Some(rl) = wrapper.take_readline()
        {
            Some((wrapper, rl))
        } else {
            None
        };

        let result = if let Some((_, ref mut rl)) = rl_taken {
            rl.readline(&prompt).is_ok_and(|line| {
                let input = line.trim().to_lowercase();
                input == "y" || input == "yes"
            })
        } else {
            warn!(
                "Console readline is not available; cannot prompt for plugin \"{}\" permissions",
                metadata.name
            );
            false
        };

        if let Some((wrapper, rl)) = rl_taken {
            wrapper.return_readline(rl);
        }

        (result, start_time.elapsed(), true)
    }

    /// Spawn initialization for a single plugin
    #[allow(clippy::too_many_lines)]
    async fn spawn_plugin_initialization(
        self: &Arc<Self>,
        server: Arc<Server>,
        instance: Arc<dyn Plugin>,
        metadata: PluginMetadata,
        loader_data: Box<dyn Any + Send + Sync>,
        loader: Arc<dyn PluginLoader>,
        path: PathBuf,
    ) -> Result<tokio::task::JoinHandle<()>, ManagerError> {
        // Mark plugin as loading
        self.plugin_states
            .write()
            .await
            .insert(metadata.name.clone(), PluginState::Loading);

        let context = Arc::new(Context::new(
            metadata.clone(),
            server.clone(),
            Arc::clone(self),
            Arc::clone(&LOGGER_IMPL),
        ));

        // Create the plugin structure first
        let plugin = LoadedPlugin {
            metadata: metadata.clone(),
            instance: None, // Will be set after successful initialization
            loader: loader.clone(),
            loader_data: Some(loader_data),
            is_active: false, // Will be set to true after successful initialization
            context: context.clone(),
            path,
        };

        let plugin_index = {
            let mut plugins = self
                .plugins
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plugins.push(plugin);
            plugins.len() - 1
        };

        // Spawn async task for plugin initialization
        let self_ref_clone = Arc::clone(self);
        let state_notify = Arc::clone(&self.state_notify);
        let plugin_name = metadata.name.clone();
        let loader_clone = loader.clone();

        let task = server.spawn_task(async move {
            // Initialize the plugin
            match instance.on_load(context.clone()).await {
                Ok(()) => {
                    // Update plugin state to loaded
                    {
                        let mut plugins = self_ref_clone
                            .plugins
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
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

                    info!("Loaded {} ({})", metadata.name, metadata.version);

                    if !metadata.permissions.is_empty() {
                        warn!(
                            "Plugin \"{}\" uses the following permissions: {:?}",
                            metadata.name, metadata.permissions
                        );
                    }
                }
                Err(e) => {
                    // Handle initialization failure
                    let error_msg = format!("Initialization failed: {e}");
                    let _ = instance.on_unload(context).await;

                    // Get the loader data before removing the plugin
                    let loader_data: Option<Box<dyn Any + Send + Sync>> = {
                        let mut plugins = self_ref_clone
                            .plugins
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if let Some(plugin) = plugins.get_mut(plugin_index) {
                            plugin.loader_data.take()
                        } else {
                            None
                        }
                    };

                    // Try to unload the plugin data
                    if let Some(data) = loader_data {
                        loader_clone.unload(data).await.ok();
                    }

                    {
                        let mut plugins = self_ref_clone
                            .plugins
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if plugin_index < plugins.len() {
                            plugins.remove(plugin_index);
                        }
                    };
                    self_ref_clone
                        .plugin_states
                        .write()
                        .await
                        .insert(plugin_name.clone(), PluginState::Failed(error_msg.clone()));
                    state_notify.notify_waiters();

                    error!("Failed to initialize plugin {plugin_name}: {error_msg}",);
                }
            }
        });

        Ok(task)
    }

    /// Load all plugins from the plugin directory
    #[allow(clippy::too_many_lines)]
    pub async fn load_plugins(
        self: &Arc<Self>,
        server: &Arc<Server>,
    ) -> Result<std::time::Duration, ManagerError> {
        let path = Path::new(PLUGIN_DIR);

        if !path.exists() {
            std::fs::create_dir(path)?;
            return Ok(std::time::Duration::ZERO);
        }

        let cache_path = path.join("permission_cache.json");
        let mut cache = cache::PermissionCache::load(&cache_path).await;

        let mut prepared_plugins = Vec::new();
        let loaders = self.loaders.read().await.clone();

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("deactivated"))
            {
                continue;
            }

            // Find a loader that can handle this file
            let mut loader_found = false;
            for loader in &loaders {
                if loader.can_load(&path) {
                    match loader.load(&path).await {
                        Ok((instance, metadata, loader_data)) => {
                            let plugin_override =
                                server.advanced_config.plugins.overrides.get(&metadata.name);

                            if plugin_override.is_some_and(|o| !o.enabled) {
                                info!(
                                    "Plugin \"{}\" is disabled in configuration, skipping.",
                                    metadata.name
                                );
                                loader_found = true;
                                break;
                            }

                            let allow_unsigned = plugin_override
                                .and_then(|o| o.allow_unsigned)
                                .unwrap_or(server.advanced_config.plugins.allow_unsigned);

                            if !allow_unsigned && !loader.is_signed(&path) {
                                error!(
                                    "Plugin \"{}\" ({:?}) is unsigned or invalid and allow_unsigned is disabled in configuration, skipping.",
                                    metadata.name, path
                                );
                                loader_found = true;
                                break;
                            }

                            prepared_plugins.push((
                                instance,
                                metadata,
                                loader_data,
                                loader.clone(),
                                path.clone(),
                            ));
                            loader_found = true;
                        }
                        Err(err) => error!("Failed to load plugin from {:?}: {}", path, err),
                    }
                    break;
                }
            }

            if !loader_found {
                self.unloaded_files.write().await.insert(path.clone());
            }
        }

        // Resolve dependencies
        let metadata_list: Vec<(String, Vec<String>)> = prepared_plugins
            .iter()
            .map(|(_, m, _, _, _)| (m.name.clone(), m.dependencies.clone()))
            .collect();

        let sorted_names =
            Self::topological_sort(&metadata_list).map_err(ManagerError::DependencyError)?;

        // Map names back to prepared plugins
        #[expect(clippy::type_complexity)]
        let mut plugins_map: HashMap<
            String,
            (
                Arc<dyn Plugin>,
                PluginMetadata,
                Box<dyn Any + Send + Sync>,
                Arc<dyn PluginLoader>,
                PathBuf,
            ),
        > = prepared_plugins
            .into_iter()
            .map(|(i, m, d, l, p)| (m.name.clone(), (i, m, d, l, p)))
            .collect();

        let mut total_wait_time = std::time::Duration::ZERO;

        for name in sorted_names {
            if let Some((instance, metadata, loader_data, loader, path)) = plugins_map.remove(&name)
            {
                let (allowed, wait_time) = self
                    .clone()
                    .check_permissions_cached(&path, &metadata, &mut cache, &cache_path, server)
                    .await;

                total_wait_time += wait_time;

                if !allowed {
                    warn!(
                        "Permission denied for plugin \"{}\", skipping loading.",
                        metadata.name
                    );
                    continue;
                }

                match self
                    .spawn_plugin_initialization(
                        server.clone(),
                        instance,
                        metadata,
                        loader_data,
                        loader,
                        path,
                    )
                    .await
                {
                    Ok(task) => {
                        // We must await each initialization to ensure dependencies are ready
                        if let Err(err) = task.await {
                            error!("Plugin initialization task panicked: {}", err);
                        }
                    }
                    Err(err) => error!("{}", err),
                }
            }
        }

        Ok(total_wait_time)
    }

    async fn check_permissions_cached(
        &self,
        path: &Path,
        metadata: &PluginMetadata,
        cache: &mut cache::PermissionCache,
        cache_path: &Path,
        server: &Arc<Server>,
    ) -> (bool, std::time::Duration) {
        let plugin_config = &server.advanced_config.plugins;
        let plugin_override = plugin_config.overrides.get(&metadata.name);

        let is_blocked = |p: &str| {
            plugin_config.blocked_permissions.iter().any(|b| b == p)
                || plugin_override.is_some_and(|o| o.blocked_permissions.iter().any(|b| b == p))
        };

        let is_pre_allowed = |p: &str| {
            plugin_config.allowed_permissions.iter().any(|a| a == p)
                || plugin_override.is_some_and(|o| o.allowed_permissions.iter().any(|a| a == p))
        };

        let effective_permissions: Vec<String> = metadata
            .permissions
            .iter()
            .filter(|p| !is_blocked(p))
            .cloned()
            .collect();

        // If all requested permissions are pre-allowed, grant without prompting
        if !effective_permissions.iter().any(|p| !is_pre_allowed(p)) {
            return (true, std::time::Duration::ZERO);
        }

        let hash = cache::calculate_hash(path).await.unwrap_or_default();

        if let Some(entry) = cache.entries.get(&hash)
            && entry.permissions_requested == metadata.permissions
        {
            info!(
                "Found cached permission decision for plugin \"{}\" (approved: {})",
                metadata.name, entry.approved
            );
            return (entry.approved, std::time::Duration::ZERO);
        }

        if !plugin_config.ask_permission_confirmation {
            info!(
                "Auto-approving permissions for plugin \"{}\" (ask_permission_confirmation is disabled)",
                metadata.name
            );
            cache.entries.insert(
                hash,
                cache::PermissionCacheEntry {
                    permissions_requested: metadata.permissions.clone(),
                    approved: true,
                },
            );
            let _ = cache.save(cache_path).await;
            return (true, std::time::Duration::ZERO);
        }

        let (allowed, wait_time, cacheable) = self.ask_permission_confirmation(metadata);
        if cacheable {
            cache.entries.insert(
                hash,
                cache::PermissionCacheEntry {
                    permissions_requested: metadata.permissions.clone(),
                    approved: allowed,
                },
            );
            let _ = cache.save(cache_path).await;
        }
        (allowed, wait_time)
    }

    /// Start loading a plugin asynchronously
    async fn start_loading_plugin(
        self: &Arc<Self>,
        server: &Arc<Server>,
        path: &Path,
    ) -> Result<tokio::task::JoinHandle<()>, ManagerError> {
        if !server.advanced_config.plugins.enabled {
            return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                "Plugin system is disabled in configuration".to_string(),
            )));
        }

        let loaders = self.loaders.read().await.clone();
        for loader in &loaders {
            if loader.can_load(path) {
                let (instance, metadata, loader_data) = loader.load(path).await?;

                let plugin_override = server.advanced_config.plugins.overrides.get(&metadata.name);

                if plugin_override.is_some_and(|o| !o.enabled) {
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        format!("Plugin \"{}\" is disabled in configuration", metadata.name),
                    )));
                }

                let allow_unsigned = plugin_override
                    .and_then(|o| o.allow_unsigned)
                    .unwrap_or(server.advanced_config.plugins.allow_unsigned);

                if !allow_unsigned && !loader.is_signed(path) {
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        format!(
                            "Plugin \"{}\" is unsigned or invalid and allow_unsigned is disabled",
                            metadata.name
                        ),
                    )));
                }

                let cache_path = Path::new(PLUGIN_DIR).join("permission_cache.json");
                let mut cache = cache::PermissionCache::load(&cache_path).await;

                let (allowed, _) = self
                    .check_permissions_cached(path, &metadata, &mut cache, &cache_path, server)
                    .await;

                if !allowed {
                    warn!(
                        "Permission denied for plugin \"{}\", skipping loading.",
                        metadata.name
                    );
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        "Permission denied".to_string(),
                    )));
                }

                return self
                    .spawn_plugin_initialization(
                        server.clone(),
                        instance,
                        metadata,
                        loader_data,
                        loader.clone(),
                        path.to_path_buf(),
                    )
                    .await;
            }
        }

        // No loader could handle this file, track it for future attempts
        self.unloaded_files.write().await.insert(path.to_path_buf());

        Err(ManagerError::PluginNotFound(
            path.to_string_lossy().to_string(),
        ))
    }

    /// Attempt to load a single plugin file
    pub async fn try_load_plugin(
        self: &Arc<Self>,
        server: &Arc<Server>,
        path: &Path,
    ) -> Result<(), ManagerError> {
        self.start_loading_plugin(server, path)
            .await?
            .await
            .map_err(|e| {
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
                            LoaderError::InitializationFailed(error),
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
    pub fn is_plugin_active(&self, name: &str) -> bool {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins
            .iter()
            .any(|p| p.metadata.name == name && p.is_active && p.instance.is_some())
    }

    /// Get list of active plugins
    #[must_use]
    pub fn active_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins
            .iter()
            .filter(|p| p.is_active && p.instance.is_some())
            .map(|p| p.metadata.clone())
            .collect()
    }

    /// Get telemetry information for all active plugins.
    #[must_use]
    pub fn active_plugins_telemetry(&self) -> Vec<crate::telemetry::PluginInfo> {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins
            .iter()
            .filter(|p| p.is_active && p.instance.is_some())
            .map(|p| crate::telemetry::PluginInfo {
                name: p.metadata.name.clone(),
                version: p.metadata.version.clone(),
                language: p
                    .loader_data
                    .as_deref()
                    .and_then(|data| p.loader.binding_language(data)),
            })
            .collect()
    }

    /// Checks if plugin loaded
    #[must_use]
    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins.iter().any(|p| p.metadata.name == name)
    }

    /// Get list of loaded plugins
    #[must_use]
    pub fn loaded_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins.iter().map(|p| p.metadata.clone()).collect()
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&self, name: &str) -> Result<(), ManagerError> {
        let mut plugin = {
            let mut plugins = self
                .plugins
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let index = plugins
                .iter()
                .position(|p| p.metadata.name == name)
                .ok_or_else(|| ManagerError::PluginNotFound(name.to_string()))?;
            plugins.remove(index)
        };

        self.unregister_handlers(name);
        self.java_versions.unregister(name);
        self.bedrock_versions.unregister(name);
        plugin.context.unregister_commands();

        if let Some(instance) = plugin.instance.take() {
            instance.on_unload(plugin.context.clone()).await.ok();
        }

        if plugin.loader.can_unload() {
            if let Some(data) = plugin.loader_data {
                plugin.loader.unload(data).await?;
            }
        } else {
            plugin.is_active = false;
            self.plugins
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(plugin);
        }

        // Remove from plugin states
        self.plugin_states.write().await.remove(name);

        Ok(())
    }

    pub(crate) fn unregister_handlers(&self, source: &str) {
        self.handlers.rcu(|handlers| {
            let mut new_handlers = (**handlers).clone();
            new_handlers.retain(|_, handlers| {
                handlers.retain(|handler| handler.source() != Some(source));
                !handlers.is_empty()
            });
            Arc::new(new_handlers)
        });
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

    /// Register a non-blocking/concurrently-ran event handler that may
    /// only read, but not modify, the event.
    pub fn register_nonblocking<E, H>(
        &self,
        handler: Arc<H>,
        priority: EventPriority,
        source: Option<String>,
    ) where
        E: Payload + Send + Sync + 'static,
        H: EventRoHandler<E> + 'static,
    {
        let typed_handler: Arc<dyn DynEventHandler> = Arc::new(TypedEventRoHandler {
            handler,
            priority,
            source,
            _phantom: std::marker::PhantomData,
        });

        self.register_dyn(E::get_name_static(), &typed_handler);
    }

    /// Register a blocking/sequentially-ran event handler that may
    /// both read and write events.
    pub fn register_blocking<E, H>(
        &self,
        handler: Arc<H>,
        priority: EventPriority,
        source: Option<String>,
    ) where
        E: Payload + Send + Sync + 'static,
        H: EventRwHandler<E> + 'static,
    {
        let typed_handler: Arc<dyn DynEventHandler> = Arc::new(TypedEventRwHandler {
            handler,
            priority,
            source,
            _phantom: std::marker::PhantomData,
        });

        self.register_dyn(E::get_name_static(), &typed_handler);
    }

    /// Register a dynamic event handler, that may be either blocking or
    /// unblocking, and that consumes a raw `QueuedEvent` handle.
    pub fn register_dyn(&self, event_type: &'static str, handler: &Arc<dyn DynEventHandler>) {
        self.handlers.rcu(|handlers| {
            let mut new_handlers = (**handlers).clone();
            new_handlers
                .entry(event_type)
                .or_default()
                .push(handler.clone());
            Arc::new(new_handlers)
        });
    }

    #[must_use]
    pub fn has_handlers<E: Payload + 'static>(&self) -> bool {
        self.handlers
            .load()
            .get(E::get_name_static())
            .is_some_and(|handlers| !handlers.is_empty())
    }

    #[expect(
        clippy::expect_used,
        reason = "Only the typed dispatch wrapper takes the payload, after handlers finish."
    )]
    async fn dispatch_event<E: Payload + 'static>(
        server: &Arc<Server>,
        event: E,
        handlers: &[Arc<dyn DynEventHandler>],
    ) -> E {
        let slot = Arc::new(RwLock::new(Some(event)));

        for handler in handlers.iter().filter(|handler| handler.is_blocking()) {
            let slot_clone: Arc<RwLock<dyn EventSlot>> = slot.clone();

            handler
                .handle_dyn(
                    server,
                    QueuedEvent(QueuedEventInner::ReadWrite(slot_clone.write_owned().await)),
                )
                .await;
        }

        join_all(
            handlers
                .iter()
                .filter(|handler| !handler.is_blocking())
                .map(|handler| {
                    let slot_clone: Arc<RwLock<dyn EventSlot>> = slot.clone();
                    async {
                        handler
                            .handle_dyn(
                                server,
                                QueuedEvent(QueuedEventInner::ReadOnly(
                                    slot_clone.read_owned().await,
                                )),
                            )
                            .await;
                    }
                }),
        )
        .await;

        slot.write().await.take().expect("dispatch owns the event")
    }

    /// Returns the event after blocking handlers and notifications have finished.
    pub async fn fire<E: Payload + 'static>(&self, server: &Arc<Server>, event: E) -> E {
        let snapshot = self.handlers.load();
        let Some(handlers) = snapshot
            .get(E::get_name_static())
            .filter(|handlers| !handlers.is_empty())
        else {
            return event;
        };
        Self::dispatch_event(server, event, handlers).await
    }

    /// Runs handlers synchronously, returning immediately when none are registered.
    #[expect(
        clippy::expect_used,
        reason = "The pinned future stores the event before returning."
    )]
    pub fn fire_blocking<E: Payload + 'static>(&self, server: &Arc<Server>, event: E) -> E {
        let snapshot = self.handlers.load();
        let Some(handlers) = snapshot
            .get(E::get_name_static())
            .filter(|handlers| !handlers.is_empty())
        else {
            return event;
        };

        let mut result = None;
        Self::block_on_dispatch(
            server,
            pin!(async {
                result = Some(Self::dispatch_event(server, event, handlers).await);
            }),
        );
        result.expect("blocking dispatch returned the event")
    }

    fn block_on_dispatch(server: &Arc<Server>, dispatch: Pin<&mut dyn Future<Output = ()>>) {
        let dispatch = || server.runtime.block_on(dispatch);
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(dispatch);
        } else {
            dispatch();
        }
    }

    #[expect(clippy::result_unit_err)]
    pub async fn send_message(
        &self,
        sender: &str,
        recipient: &str,
        message: &[u8],
    ) -> Result<Result<Vec<u8>, String>, ()> {
        if sender == recipient {
            return Err(());
        }

        let instance = {
            let plugins = self
                .plugins
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let target_plugin = plugins
                .iter()
                .find(|p| p.metadata.name == recipient)
                .ok_or(())?;
            target_plugin.instance.clone()
        };
        if let Some(instance) = instance {
            Ok(instance.on_ipc_message(sender, message).await)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::plugin::events::server::ServerTickStartEvent;

    #[tokio::test]
    async fn queued_event_survives_caller_cancellation() {
        let (submitted, queued) = tokio::sync::oneshot::channel();
        let caller = tokio::spawn(async move {
            let slot = Arc::new(RwLock::new(Some(ServerTickStartEvent::new(3))));
            let slot_clone: Arc<RwLock<dyn EventSlot>> = slot.clone();
            let handle = QueuedEvent(QueuedEventInner::ReadWrite(
                slot_clone.try_write_owned().unwrap(),
            ));
            assert!(submitted.send(handle).is_ok());
            std::future::pending::<()>().await;
            drop(slot);
        });

        let mut handle = queued.await.unwrap();
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        handle.write::<ServerTickStartEvent>().unwrap().tick = 4;
        assert_eq!(handle.read::<ServerTickStartEvent>().unwrap().tick, 4);
    }

    #[tokio::test]
    async fn queued_event_shares_edits_and_expires_after_dispatch() {
        let slot = Arc::new(RwLock::new(Some(ServerTickStartEvent::new(1))));

        {
            let mut blocking = QueuedEvent(QueuedEventInner::ReadWrite(
                (slot.clone() as Arc<RwLock<dyn EventSlot>>)
                    .write_owned()
                    .await,
            ));

            blocking.write::<ServerTickStartEvent>().unwrap().tick = 2;
        };

        {
            let mut notification = QueuedEvent(QueuedEventInner::ReadOnly(
                (slot.clone() as Arc<RwLock<dyn EventSlot>>)
                    .read_owned()
                    .await,
            ));

            assert!(notification.write::<ServerTickStartEvent>().is_none());
            assert_eq!(notification.read::<ServerTickStartEvent>().unwrap().tick, 2);
        };

        {
            let mut guard = slot.write().await;
            let event = guard.payload_mut().unwrap();
            event
                .as_any_mut()
                .downcast_mut::<ServerTickStartEvent>()
                .unwrap()
                .tick = 3;
        };

        {
            let guard = slot.read().await;
            let event = guard.payload().unwrap();
            assert_eq!(
                event
                    .as_any()
                    .downcast_ref::<ServerTickStartEvent>()
                    .unwrap()
                    .tick,
                3
            );
        };

        {
            let notification = QueuedEvent(QueuedEventInner::ReadOnly(
                (slot.clone() as Arc<RwLock<dyn EventSlot>>)
                    .read_owned()
                    .await,
            ));

            assert_eq!(notification.read::<ServerTickStartEvent>().unwrap().tick, 3);
        };

        assert_eq!(slot.write().await.take().unwrap().tick, 3);

        {
            let notification = QueuedEvent(QueuedEventInner::ReadOnly(
                (slot.clone() as Arc<RwLock<dyn EventSlot>>)
                    .read_owned()
                    .await,
            ));
            assert!(notification.read::<ServerTickStartEvent>().is_none());
        };

        {
            let mut blocking = QueuedEvent(QueuedEventInner::ReadWrite(
                (slot.clone() as Arc<RwLock<dyn EventSlot>>)
                    .write_owned()
                    .await,
            ));

            assert!(blocking.write::<ServerTickStartEvent>().is_none());
        };
        assert!(slot.read().await.payload().is_none());
    }

    #[tokio::test]
    async fn topological_sort() {
        let plugins = vec![
            ("A".to_string(), vec!["B".to_string()]),
            ("B".to_string(), vec!["C".to_string()]),
            ("C".to_string(), vec![]),
        ];
        let sorted = PluginManager::topological_sort(&plugins).unwrap();
        assert_eq!(sorted, vec!["C", "B", "A"]);

        let plugins_complex = vec![
            ("A".to_string(), vec!["B".to_string(), "C".to_string()]),
            ("B".to_string(), vec!["D".to_string()]),
            ("C".to_string(), vec!["D".to_string()]),
            ("D".to_string(), vec![]),
        ];
        let sorted = PluginManager::topological_sort(&plugins_complex).unwrap();
        // Multiple valid sorts possible, but D must be before B and C, and B, C must be before A.
        assert_eq!(sorted[0], "D");
        assert!(sorted[1] == "B" || sorted[1] == "C");
        assert!(sorted[2] == "B" || sorted[2] == "C");
        assert_eq!(sorted[3], "A");

        let plugins_circular = vec![
            ("A".to_string(), vec!["B".to_string()]),
            ("B".to_string(), vec!["A".to_string()]),
        ];
        assert!(PluginManager::topological_sort(&plugins_circular).is_err());

        let plugins_missing = vec![("A".to_string(), vec!["B".to_string()])];
        assert!(PluginManager::topological_sort(&plugins_missing).is_err());
    }
}
