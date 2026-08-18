use futures::future::join_all;
use loader::{LoaderError, PluginLoader, native::NativePluginLoader};
use notify::{EventKind, RecursiveMode, Watcher, event::ModifyKind};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use thiserror::Error;
use tokio::{
    sync::{Notify, RwLock},
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

use crate::{LOGGER_IMPL, plugin::loader::wasm::WasmPluginLoader, server::Server};
pub use api::*;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Bump this whenever the public plugin API or any event layout changes in a way
/// that makes old binary plugins incompatible.
pub const PLUGIN_API_VERSION: u32 = 2;

const PLUGIN_DIR: &str = "./plugins";

/// A trait for handling events dynamically.
///
/// This trait allows for handling events of any type that implements the `Event` trait.
pub trait DynEventHandler: Send + Sync {
    /// Asynchronously handles a dynamic event.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    fn handle_dyn<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a (dyn Payload + Send + Sync),
    ) -> BoxFuture<'a, ()>;

    /// Asynchronously handles a blocking dynamic event.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    fn handle_blocking_dyn<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut (dyn Payload + Send + Sync),
    ) -> BoxFuture<'a, ()>;

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
}

/// A trait for handling specific events.
///
/// This trait allows for handling events of a specific type that implements the `Event` trait.
pub trait EventHandler<E: Payload>: Send + Sync {
    /// Asynchronously handles an event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A reference to the event to handle.
    fn handle<'a>(&'a self, _server: &'a Arc<Server>, _event: &'a E) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }

    /// Asynchronously handles a blocking event of type `E`.
    ///
    /// # Arguments
    /// - `event`: A mutable reference to the event to handle.
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut E,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

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
    handlers: Arc<RwLock<HandlerMap>>,
    unloaded_files: RwLock<HashSet<PathBuf>>,
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
        Self::new()
    }
}

impl PluginManager {
    /// Create a new plugin manager with default loaders
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(Vec::new()),
            loaders: RwLock::new(vec![
                Arc::new(NativePluginLoader),
                Arc::new(WasmPluginLoader),
            ]),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            unloaded_files: RwLock::new(HashSet::new()),
            services: Arc::new(RwLock::new(HashMap::new())),
            plugin_states: RwLock::new(HashMap::new()),
            state_notify: Arc::new(Notify::new()),
            hot_reload_task: RwLock::new(None),
            hot_reload_enabled: AtomicBool::new(false),
        }
    }

    /// Unload all loaded plugins
    pub async fn unload_all_plugins(&self) -> Result<(), ManagerError> {
        let plugin_names: Vec<String> = {
            let plugins = self.plugins.read().await;
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
        let server = server.clone();
        let task = tokio::spawn(async move {
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
                                    let plugins = manager.plugins.read().await;
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
                                if let Err(e) = manager.start_loading_plugin(&server, &path).await {
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
    #[expect(clippy::print_stdout)]
    fn ask_permission_confirmation(metadata: &PluginMetadata) -> (bool, std::time::Duration) {
        use colored::Colorize;
        use rustyline::DefaultEditor;

        if metadata.permissions.is_empty() {
            return (true, std::time::Duration::ZERO);
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
        } else if let Ok(mut rl) = DefaultEditor::new() {
            rl.readline(&prompt).is_ok_and(|line| {
                let input = line.trim().to_lowercase();
                input == "y" || input == "yes"
            })
        } else {
            false
        };

        if let Some((wrapper, rl)) = rl_taken {
            wrapper.return_readline(rl);
        }

        (result, start_time.elapsed())
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
            server,
            Arc::clone(&self.handlers),
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
            let mut plugins = self.plugins.write().await;
            plugins.push(plugin);
            plugins.len() - 1
        };

        // Spawn async task for plugin initialization
        let self_ref_clone = Arc::clone(self);
        let state_notify = Arc::clone(&self.state_notify);
        let plugin_name = metadata.name.clone();
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
            let loaders = self.loaders.read().await;
            let mut loader_found = false;
            for loader in loaders.iter() {
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

                            if !allow_unsigned
                                && path
                                    .extension()
                                    .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm"))
                            {
                                let wasm_bytes = std::fs::read(&path).unwrap_or_default();
                                if !crate::plugin::loader::wasm::wasm_host::signature::is_wasm_signed(&wasm_bytes) {
                                    error!(
                                        "Plugin \"{}\" ({:?}) is unsigned or invalid and allow_unsigned is disabled in configuration, skipping.",
                                        metadata.name, path
                                    );
                                    loader_found = true;
                                    break;
                                }
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

        let (allowed, wait_time) = Self::ask_permission_confirmation(metadata);
        cache.entries.insert(
            hash,
            cache::PermissionCacheEntry {
                permissions_requested: metadata.permissions.clone(),
                approved: allowed,
            },
        );
        let _ = cache.save(cache_path).await;
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

        for loader in self.loaders.read().await.iter() {
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

                if !allow_unsigned
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("wasm"))
                {
                    let wasm_bytes = std::fs::read(path).unwrap_or_default();
                    if !crate::plugin::loader::wasm::wasm_host::signature::is_wasm_signed(
                        &wasm_bytes,
                    ) {
                        return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                            format!(
                                "Plugin \"{}\" is unsigned or invalid and allow_unsigned is disabled",
                                metadata.name
                            ),
                        )));
                    }
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
    pub async fn is_plugin_active(&self, name: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins
            .iter()
            .any(|p| p.metadata.name == name && p.is_active && p.instance.is_some())
    }

    /// Get list of active plugins
    #[must_use]
    pub async fn active_plugins(&self) -> Vec<PluginMetadata> {
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
    pub async fn loaded_plugins(&self) -> Vec<PluginMetadata> {
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

        if let Some(instance) = plugin.instance.take() {
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
        E: Payload + Send + Sync + 'static,
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
    pub async fn fire<E: Payload + Send + Sync + 'static>(
        &self,
        server: &Arc<Server>,
        event: &mut E,
    ) {
        let handlers_lock = self.handlers.read().await;
        if handlers_lock.is_empty() {
            return;
        }

        let Some(handlers) = handlers_lock.get(&E::get_name_static()) else {
            return;
        };

        if handlers.is_empty() {
            return;
        }

        // Process blocking handlers first
        for handler in handlers {
            if handler.is_blocking() {
                handler.handle_blocking_dyn(server, event).await;
            }
        }

        // Process non-blocking handlers
        for handler in handlers {
            if !handler.is_blocking() {
                handler.handle_dyn(server, event).await;
            }
        }
    }

    pub async fn send_message(
        &self,
        sender: &str,
        recipient: &str,
        message: &[u8],
    ) -> Result<Result<Vec<u8>, String>, ()> {
        if sender == recipient {
            return Err(());
        }

        let plugins = self.plugins.read().await;
        let target_plugin = &plugins
            .iter()
            .find(|p| p.metadata.name == recipient)
            .ok_or(())?;
        if let Some(instance) = &target_plugin.instance {
            Ok(instance.on_ipc_message(sender, message).await)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
