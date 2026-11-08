use arc_swap::ArcSwap;
use futures::{FutureExt, future::join_all};
use loader::{LoaderError, PluginLoader, native::NativePluginLoader};
use notify::{EventKind, RecursiveMode, Watcher, event::ModifyKind};
use std::{
    any::Any,
    borrow::Cow,
    collections::{HashMap, HashSet},
    panic::AssertUnwindSafe,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{
        Arc, Mutex as SyncMutex, PoisonError, RwLock as SyncRwLock,
        atomic::{AtomicBool, AtomicU64},
    },
    thread::ThreadId,
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

use crate::{LOGGER_IMPL, server::Server};

tokio::task_local! {
    /// The plugin whose `on_load` runs on this task
    static INITIALIZING_PLUGIN: PluginKey;
}

/// A plugin and the manager that owns it, since managers can hold same-named plugins.
type PluginKey = (u64, String);

/// Gives every manager its own id.
static NEXT_MANAGER_ID: AtomicU64 = AtomicU64::new(0);
pub use api::*;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Bump this whenever the public plugin API or any event layout changes in a way
/// that makes old binary plugins incompatible.
pub const PLUGIN_API_VERSION: u32 = 5;

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

    /// Returns the plugin that registered this handler, when applicable.
    fn source(&self) -> Option<&str> {
        None
    }
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
pub struct TypedEventHandler<E, H>
where
    E: Payload + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    pub handler: Arc<H>,
    pub priority: EventPriority,
    pub blocking: bool,
    pub source: Option<String>,
    pub _phantom: std::marker::PhantomData<E>,
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

/// State of a plugin file the manager knows about, running or not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Active,
    Loading,
    /// Unloaded on request
    Unloaded,
    /// Turned off in the configuration
    Disabled,
    /// Unsigned while `allow_unsigned` is off
    Unsigned,
    PermissionDenied,
    /// Loader or initialization error
    Failed(String),
    /// No loader accepts the file
    NoLoader,
}

impl PluginStatus {
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl std::fmt::Display for PluginStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => f.write_str("Active"),
            Self::Loading => f.write_str("Loading"),
            Self::Unloaded => f.write_str("Unloaded"),
            Self::Disabled => f.write_str("Disabled in the server configuration"),
            Self::Unsigned => {
                f.write_str("Unsigned or invalid signature, and allow_unsigned is off")
            }
            Self::PermissionDenied => f.write_str("Permission request denied"),
            Self::Failed(error) => write!(f, "Failed: {error}"),
            Self::NoLoader => f.write_str("No plugin loader accepts this file"),
        }
    }
}

/// A plugin file, running or not.
#[derive(Debug, Clone)]
pub struct PluginEntry {
    /// `None` when the file never got far enough to name itself
    pub metadata: Option<PluginMetadata>,
    pub path: PathBuf,
    pub status: PluginStatus,
    /// False while the loader holds it and cannot release it at runtime
    pub can_unload: bool,
}

impl PluginEntry {
    /// The plugin name, or the file name when it never named itself.
    #[must_use]
    pub fn name(&self) -> Cow<'_, str> {
        self.metadata.as_ref().map_or_else(
            || self.path.file_name().unwrap_or_default().to_string_lossy(),
            |metadata| Cow::Borrowed(metadata.name.as_str()),
        )
    }
}

/// Keeps the first, richer record when a later source names the same file again.
///
/// Another file declaring a known name stays listed.
fn push_unless_known(entries: &mut Vec<PluginEntry>, entry: PluginEntry) {
    if !entries
        .iter()
        .any(|known| known.name() == entry.name() && same_file(&known.path, &entry.path))
    {
        entries.push(entry);
    }
}

/// Equal paths, or two spellings of one file.
fn same_file(a: &Path, b: &Path) -> bool {
    a == b || std::fs::canonicalize(a).is_ok_and(|a| std::fs::canonicalize(b).is_ok_and(|b| a == b))
}

/// Whether a file claims to be a plugin, by extension.
fn is_plugin_file(path: &Path) -> bool {
    path.extension().is_some_and(|ext| {
        ["wasm", "so", "dll", "dylib"]
            .iter()
            .any(|known| ext.eq_ignore_ascii_case(known))
    })
}

/// Server permission namespaces, never a plugin name.
const RESERVED_NAMESPACES: [&str; 2] = ["minecraft", "pumpkin"];

/// How long an unload waits for events still running the plugin's handlers.
const HANDLER_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);
const HANDLER_DRAIN_POLL: Duration = Duration::from_millis(10);

/// A registered service and the plugin that registered it.
pub(crate) struct Service {
    pub(crate) owner: String,
    pub(crate) payload: Arc<dyn Payload>,
}

/// Services by name, plus replaced ones a holder may still use.
#[derive(Default)]
pub(crate) struct Services {
    active: HashMap<String, Service>,
    /// Off the map, still drained with their owner.
    replaced: Vec<Service>,
}

impl Services {
    pub(crate) fn insert(&mut self, name: String, service: Service) {
        // Only our count left: owner still loaded, safe to drop
        self.replaced
            .retain(|old| Arc::strong_count(&old.payload) > 1);
        if let Some(old) = self.active.insert(name, service) {
            self.replaced.push(old);
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Arc<dyn Payload>> {
        self.active.get(name).map(|service| &service.payload)
    }

    /// Takes every service `owner` registered, replaced ones included.
    fn take_owned(&mut self, owner: &str) -> Vec<Arc<dyn Payload>> {
        let mut taken = Vec::new();
        let mut take = |service: &Service| {
            let keep = service.owner != owner;
            if !keep {
                taken.push(Arc::clone(&service.payload));
            }
            keep
        };
        self.active.retain(|_, service| take(service));
        self.replaced.retain(|service| take(service));
        taken
    }
}

/// A plugin loader and the plugin that registered it, `None` for the server's own.
struct RegisteredLoader {
    owner: Option<String>,
    loader: Arc<dyn PluginLoader>,
}

/// Handlers, services and loaders taken off a plugin. Their code lives in its library.
struct Registrations {
    handlers: Vec<Arc<dyn DynEventHandler>>,
    services: Vec<Arc<dyn Payload>>,
    loaders: Vec<Arc<dyn PluginLoader>>,
}

impl Registrations {
    /// Any count above ours: a running event, a service holder, or a plugin from that loader.
    fn in_use(&self) -> bool {
        self.handlers.iter().any(|h| Arc::strong_count(h) > 1)
            || self.services.iter().any(|s| Arc::strong_count(s) > 1)
            || self.loaders.iter().any(|l| Arc::strong_count(l) > 1)
    }

    fn extend(&mut self, other: Self) {
        self.handlers.extend(other.handlers);
        self.services.extend(other.services);
        self.loaders.extend(other.loaders);
    }
}

/// Hands a plugin's library back to its loader once nothing runs its handlers, holds its
/// services, or holds its instance.
///
/// `commands`: its commands stay in the command tree, the library is kept right away.
/// A loader without `can_unload` keeps it right away too, without an error.
///
/// Anything still running or held after `HANDLER_DRAIN_TIMEOUT` keeps the library
/// loaded for good, releasing it would pull the code out from under that call. That is
/// `LibraryInUse`.
async fn release_library(
    name: &str,
    registrations: Registrations,
    instance: Option<Arc<dyn Plugin>>,
    loader: &dyn PluginLoader,
    loader_data: Box<dyn Any + Send + Sync>,
    commands: bool,
) -> Result<(), LoaderError> {
    // Loader keeps the library: untracked plugin work can still run its code
    if !loader.can_unload() {
        drop(registrations);
        drop(instance);
        Box::leak(loader_data);
        return Ok(());
    }
    // Tree never drops nodes: executors and argument types run its code
    // TODO: remove a plugin's nodes from the command tree after reload
    if commands {
        drop(registrations);
        drop(instance);
        warn!("Plugin {name} registered commands, keeping its library loaded");
        Box::leak(loader_data);
        return Err(LoaderError::CommandsResident);
    }
    // Each handler is wrapped once per registration, so any count above ours is a running event.
    // The same goes for the instance, which `send_message` holds across `on_ipc_message`
    let drained = tokio::time::timeout(HANDLER_DRAIN_TIMEOUT, async {
        while registrations.in_use()
            || instance
                .as_ref()
                .is_some_and(|instance| Arc::strong_count(instance) > 1)
        {
            tokio::time::sleep(HANDLER_DRAIN_POLL).await;
        }
    })
    .await
    .is_ok();
    // Their code lives in the library, so they go first
    drop(registrations);
    drop(instance);
    if drained {
        loader.unload(loader_data).await
    } else {
        warn!(
            "Plugin {name} still has event handlers, services or IPC calls in use, keeping its library loaded"
        );
        Box::leak(loader_data);
        Err(LoaderError::LibraryInUse)
    }
}

/// An `on_load` waiting on another plugin's initialization, for as long as the wait runs.
struct InitWait<'a> {
    waits: &'a SyncMutex<Vec<(PluginKey, PluginKey)>>,
    edge: (PluginKey, PluginKey),
}

impl Drop for InitWait<'_> {
    fn drop(&mut self) {
        let mut waits = self.waits.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(index) = waits.iter().position(|edge| *edge == self.edge) {
            waits.swap_remove(index);
        }
    }
}

/// Releases a plugin that was loaded but will not run.
async fn discard_loaded(
    instance: Arc<dyn Plugin>,
    loader: &dyn PluginLoader,
    loader_data: Box<dyn Any + Send + Sync>,
) {
    // The instance's code lives in the library, so it goes first
    drop(instance);
    loader.unload(loader_data).await.ok();
}

/// One load of a plugin: a later load of the same name is another one.
#[derive(Debug, Clone)]
pub struct PluginId {
    pub name: String,
    token: u64,
}

/// Core plugin management system
#[repr(C)]
pub struct PluginManager {
    plugins: SyncRwLock<Vec<LoadedPlugin>>,
    next_plugin_token: AtomicU64,
    loaders: RwLock<Vec<RegisteredLoader>>,
    pub handlers: Arc<ArcSwap<HandlerMap>>,
    unloaded_files: RwLock<HashSet<PathBuf>>,
    /// Plugin files that are known but not running, by path
    inactive: SyncRwLock<HashMap<PathBuf, PluginEntry>>,
    services: Arc<RwLock<Services>>,
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
    /// (waiting plugin, awaited plugin) for every `on_load` waiting on an initialization
    init_waits: SyncMutex<Vec<(PluginKey, PluginKey)>>,
    /// Tells this manager's plugins from same-named ones of another manager
    id: u64,
}

/// Represents a successfully loaded plugin
///
/// OS specific issues
/// - Windows: Plugin cannot be unloaded, it can be only active or not
struct LoadedPlugin {
    token: u64,
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
    #[error("Waiting on plugin {0} would deadlock, its initialization waits on the caller")]
    InitializationDeadlock(String),
}

impl ManagerError {
    /// Unload done, only the library stays loaded.
    #[must_use]
    pub const fn is_library_in_use(&self) -> bool {
        matches!(
            self,
            Self::LoaderError(LoaderError::LibraryInUse | LoaderError::CommandsResident)
        )
    }
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
        let loaders = std::iter::once(Arc::new(NativePluginLoader) as Arc<dyn PluginLoader>)
            .chain(extra_loaders)
            .map(|loader| RegisteredLoader {
                owner: None,
                loader,
            })
            .collect();
        Self {
            plugins: SyncRwLock::new(Vec::new()),
            next_plugin_token: AtomicU64::new(0),
            loaders: RwLock::new(loaders),
            handlers: Arc::new(ArcSwap::from_pointee(HashMap::new())),
            unloaded_files: RwLock::new(HashSet::new()),
            inactive: SyncRwLock::new(HashMap::new()),
            services: Arc::new(RwLock::new(Services::default())),
            plugin_states: RwLock::new(HashMap::new()),
            state_notify: Arc::new(Notify::new()),
            hot_reload_task: RwLock::new(None),
            hot_reload_enabled: AtomicBool::new(false),
            main_thread_id: std::thread::current().id(),
            init_waits: SyncMutex::new(Vec::new()),
            id: NEXT_MANAGER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Unload all loaded plugins
    pub async fn unload_all_plugins(&self) -> Result<(), ManagerError> {
        self.unload_active_plugins(|_| true).await
    }

    /// The active plugins matching `filter` with their files, dependents before what they depend on
    fn unload_order(&self, filter: impl Fn(&LoadedPlugin) -> bool) -> Vec<(String, PathBuf)> {
        let selected: Vec<(String, Vec<String>, PathBuf)> = {
            let plugins = self
                .plugins
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plugins
                .iter()
                .filter(|p| p.is_active && filter(p))
                .map(|p| {
                    (
                        p.metadata.name.clone(),
                        p.metadata.dependencies.clone(),
                        p.path.clone(),
                    )
                })
                .collect()
        };

        // Dependencies outside the selection stay loaded, so they don't take part
        let names: HashSet<&String> = selected.iter().map(|(name, _, _)| name).collect();
        let relations: Vec<(String, Vec<String>)> = selected
            .iter()
            .map(|(name, deps, _)| {
                let inside = deps.iter().filter(|dep| names.contains(dep)).cloned();
                (name.clone(), inside.collect())
            })
            .collect();
        let mut paths: HashMap<String, PathBuf> = selected
            .iter()
            .map(|(name, _, path)| (name.clone(), path.clone()))
            .collect();
        match Self::topological_sort(&relations) {
            Ok(mut order) => {
                order.reverse();
                order
                    .into_iter()
                    .filter_map(|name| paths.remove(&name).map(|path| (name, path)))
                    .collect()
            }
            Err(e) => {
                warn!("Unloading plugins in listed order: {e}");
                selected
                    .into_iter()
                    .map(|(name, _, path)| (name, path))
                    .collect()
            }
        }
    }

    /// Unload the active plugins matching `filter`, reporting the first failure
    async fn unload_active_plugins(
        &self,
        filter: impl Fn(&LoadedPlugin) -> bool,
    ) -> Result<(), ManagerError> {
        let plugin_names = self.unload_order(filter).into_iter().map(|(name, _)| name);

        // Keep unloading the rest even if one fails, but report the failure so a caller
        // like `reload_all_plugins` does not rescan while a plugin's resources are still resident
        let mut first_error = None;
        for name in plugin_names {
            match self.unload_plugin(&name).await {
                Ok(()) => {}
                // Unloaded all the same, its library is only kept
                Err(e) if e.is_library_in_use() => warn!("Plugin {name}: {e}"),
                Err(e) => {
                    error!("Failed to unload plugin {name}: {e}");
                    first_error.get_or_insert(e);
                }
            }
        }

        first_error.map_or(Ok(()), Err)
    }

    /// Unload everything that can be unloaded, then rescan the plugin directory.
    ///
    /// Picks up files added since the last scan, which is what separates it from reloading
    /// plugins one by one. Plugins whose loader can't unload stay loaded, because the rescan
    /// would skip their retained records and leave them inactive until restart.
    pub async fn reload_all_plugins(
        self: &Arc<Self>,
        server: &Arc<Server>,
    ) -> Result<(), ManagerError> {
        if !server.advanced_config.plugins.enabled {
            return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                "Plugin system is disabled in configuration".to_string(),
            )));
        }

        let unloading = self.unload_order(|p| p.loader.can_unload());
        let names: HashSet<&String> = unloading.iter().map(|(name, _)| name).collect();
        self.unload_active_plugins(|p| p.is_active && names.contains(&p.metadata.name))
            .await?;
        self.prune_missing_files().await;
        if let Err(e) = self.load_plugins(server).await {
            // The rescan failed as a whole, so bring back what was running before it
            for (name, path) in unloading.iter().rev() {
                if let Err(err) = self.try_load_plugin(server, path).await {
                    error!("Failed to restore plugin {name}: {err}");
                }
            }
            return Err(e);
        }
        Ok(())
    }

    /// Forget plugin files that are gone from disk, so a rescan stops listing them.
    async fn prune_missing_files(&self) {
        self.inactive
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|path, _| path.exists());
        self.unloaded_files
            .write()
            .await
            .retain(|path| path.exists());
    }

    /// Add a new plugin loader implementation
    pub async fn add_loader(self: &Arc<Self>, server: &Arc<Server>, loader: Arc<dyn PluginLoader>) {
        self.add_owned_loader(server, None, loader).await;
    }

    /// `owner`: plugin whose unload takes the loader off again. Refused once it unloads.
    pub(crate) async fn add_owned_loader(
        self: &Arc<Self>,
        server: &Arc<Server>,
        owner: Option<&Context>,
        loader: Arc<dyn PluginLoader>,
    ) {
        // Checked under the sweep's lock: no add can land after it
        let mut loaders = self.loaders.write().await;
        if owner.is_some_and(Context::is_closed) {
            return;
        }
        loaders.push(RegisteredLoader {
            owner: owner.map(|context| context.get_metadata().name.clone()),
            loader,
        });
        drop(loaders);

        // Try to load previously unloaded files with the new loader
        self.retry_unloaded_files(server).await;
    }

    /// Start watching the plugins directory for changes
    pub async fn start_watcher(self: &Arc<Self>, server: &Arc<Server>) -> Result<(), ManagerError> {
        // Held until the task is stored, so two starts cannot both spawn a watcher
        let mut task_slot = self.hot_reload_task.write().await;
        if task_slot.is_some() {
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

                                // Its own task: stopping the watcher must not cut an unload or
                                // load off halfway
                                let manager = Arc::clone(&manager);
                                let server = Arc::clone(&server_clone);
                                let reload = server_clone.spawn_task(async move {
                                    manager.hot_reload(&server, &path).await;
                                });
                                reload.await.ok();
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        *task_slot = Some(task);
        drop(task_slot);
        self.set_hot_reload_enabled(true);
        Ok(())
    }

    /// Replace the plugin loaded from `path` with the file's current contents.
    async fn hot_reload(self: &Arc<Self>, server: &Arc<Server>, path: &Path) {
        // We need to find if this plugin is already loaded to unload it first
        let plugin_name = {
            let plugins = self
                .plugins
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plugins
                .iter()
                .find(|p| p.path == path)
                .map(|p| p.metadata.name.clone())
        };

        let unload_failed = if let Some(name) = plugin_name {
            info!("Hot-reloading plugin: {}", name);
            match self.unload_plugin(&name).await {
                Ok(()) => false,
                Err(e) if e.is_library_in_use() => {
                    warn!("Plugin {name}: {e}");
                    false
                }
                Err(e) => {
                    error!("Failed to unload plugin {name} for hot-reload, not reloading: {e}");
                    true
                }
            }
        } else {
            false
        };

        // Skip the reload when unload failed
        // the loader may still hold the old resources, loading would create
        // a duplicate instead of a replacement.
        if !unload_failed && let Err(e) = self.start_loading_plugin(server, path).await {
            error!("Failed to hot-reload plugin {:?}: {}", path, e);
        }
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
            if let Ok((_, task)) = self.start_loading_plugin(server, &path).await {
                retry_tasks.push(task);
            }
        }

        // Wait for all retry tasks to complete
        join_all(retry_tasks).await;
    }

    /// Lets every loader run its per-tick work.
    pub fn tick_loaders(&self, server: &Arc<Server>) {
        if let Ok(loaders) = self.loaders.try_read() {
            for registered in loaders.iter() {
                registered.loader.on_tick(server);
            }
        }
    }

    /// Get a clone of the loaders for context use
    #[must_use]
    pub async fn get_loaders(&self) -> Vec<Arc<dyn PluginLoader>> {
        self.loaders
            .read()
            .await
            .iter()
            .map(|registered| Arc::clone(&registered.loader))
            .collect()
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
    ) -> Result<(u64, tokio::task::JoinHandle<Result<(), String>>), ManagerError> {
        let context = Arc::new(Context::new(
            metadata.clone(),
            server.clone(),
            Arc::clone(&self.handlers),
            Arc::clone(self),
            Arc::clone(&LOGGER_IMPL),
        ));

        let plugin_path = path.clone();

        // Unload drops every node under `{name}:`: no server namespace, no other plugin's
        let invalid = if RESERVED_NAMESPACES.contains(&metadata.name.as_str()) {
            Some("is reserved by the server")
        } else if metadata.name.contains(':') {
            Some("contains ':'")
        } else {
            None
        };
        if let Some(invalid) = invalid {
            let reason = format!("name {} {invalid}", metadata.name);
            self.mark_inactive(&path, Some(&metadata), PluginStatus::Failed(reason.clone()));
            discard_loaded(instance, &*loader, loader_data).await;
            return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                format!("Plugin \"{}\": {reason}", metadata.name),
            )));
        }

        // Claiming the name and inserting the record: two loads of
        // the same plugin would otherwise both initialize and register handlers
        let token = self
            .next_plugin_token
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // States first, then plugins, like `plugin_entries`; the Loading state is set in the same
        // section so an unload in between cannot leave it stuck
        let mut states = self.plugin_states.write().await;
        let reserved = {
            let mut plugins = self
                .plugins
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if plugins.iter().any(|p| p.metadata.name == metadata.name) {
                Err(loader_data)
            } else {
                plugins.push(LoadedPlugin {
                    token,
                    metadata: metadata.clone(),
                    instance: None, // Will be set after successful initialization
                    loader: loader.clone(),
                    loader_data: Some(loader_data),
                    is_active: false, // Will be set to true after successful initialization
                    context: context.clone(),
                    path,
                });
                states.insert(metadata.name.clone(), PluginState::Loading);
                Ok(())
            }
        };
        drop(states);

        if let Err(loader_data) = reserved {
            // Nothing owns the runtime the loader built -> release it again
            discard_loaded(instance, &*loader, loader_data).await;
            return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                format!("Plugin \"{}\" is already loaded", metadata.name),
            )));
        }

        // Spawn async task for plugin initialization
        let self_ref_clone = Arc::clone(self);
        let state_notify = Arc::clone(&self.state_notify);
        let plugin_name = metadata.name.clone();
        let manager_id = self.id;
        let loader_clone = loader.clone();

        let task = server.spawn_task(async move {
            // A panic must still reach the failure path, or the plugin stays Loading forever
            let result = INITIALIZING_PLUGIN
                .scope(
                    (manager_id, plugin_name.clone()),
                    // Called inside the block: a panic before the future exists is caught too
                    AssertUnwindSafe(async { instance.on_load(context.clone()).await })
                        .catch_unwind(),
                )
                .await
                .unwrap_or_else(|_| Err("on_load panicked".to_string()));
            match result {
                Ok(()) => {
                    // Update plugin state to loaded
                    let still_reserved = {
                        let mut plugins = self_ref_clone
                            .plugins
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if let Some(plugin) = plugins.iter_mut().find(|p| p.token == token) {
                            plugin.instance = Some(instance);
                            plugin.is_active = true;
                            true
                        } else {
                            false
                        }
                    };

                    // Unloaded while it was still initializing, so this load owns nothing
                    // any more
                    if !still_reserved {
                        warn!(
                            "Plugin {plugin_name} was unloaded while it was initializing, dropping the load"
                        );
                        state_notify.notify_waiters();
                        return Err("Unloaded while it was initializing".to_string());
                    }

                    self_ref_clone
                        .plugin_states
                        .write()
                        .await
                        .insert(plugin_name.clone(), PluginState::Loaded);
                    self_ref_clone
                        .inactive
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .remove(&plugin_path);
                    state_notify.notify_waiters();

                    info!("Loaded {} ({})", metadata.name, metadata.version);

                    if !metadata.permissions.is_empty() {
                        warn!(
                            "Plugin \"{}\" uses the following permissions: {:?}",
                            metadata.name, metadata.permissions
                        );
                    }
                    Ok(())
                }
                Err(e) => {
                    // Handle initialization failure
                    let error_msg = format!("Initialization failed: {e}");

                    if !self_ref_clone.holds_token(token) {
                        error!("Failed to initialize plugin {plugin_name}: {error_msg}");
                        state_notify.notify_waiters();
                        return Err(e);
                    }

                    // on_load may have registered handlers, commands or permissions before
                    // failing partway through, leaving them would block every future load
                    self_ref_clone.unload_children(&plugin_name).await;
                    context.close();
                    let owner = self_ref_clone.loader_owner(&loader_clone).await;
                    let mut registrations = self_ref_clone.take_registrations(&plugin_name).await;
                    context.unregister_commands();
                    context.unregister_permissions();
                    // Still Loading, so a self-unload from on_unload would wait on itself too
                    let _ = INITIALIZING_PLUGIN
                        .scope(
                            (manager_id, plugin_name.clone()),
                            AssertUnwindSafe(async { instance.on_unload(context.clone()).await })
                                .catch_unwind(),
                        )
                        .await;
                    // on_unload can register again, that must not outlive the library
                    registrations.extend(self_ref_clone.take_registrations(&plugin_name).await);
                    context.unregister_commands();
                    context.unregister_permissions();
                    self_ref_clone.pin_owner(&context, owner.as_deref());
                    // The instance's code lives in the library, so drop it before unloading
                    drop(instance);

                    // Get the loader data before removing the plugin
                    let loader_data: Option<Box<dyn Any + Send + Sync>> = {
                        let mut plugins = self_ref_clone
                            .plugins
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        plugins
                            .iter_mut()
                            .find(|p| p.token == token)
                            .and_then(|plugin| plugin.loader_data.take())
                    };

                    // Library still in use: stays leaked, the name is free again anyway
                    if let Some(data) = loader_data {
                        release_library(
                            &plugin_name,
                            registrations,
                            None,
                            &*loader_clone,
                            data,
                            context.has_commands(),
                        )
                        .await
                        .ok();
                    }

                    // Same states-then-plugins section as the reservation, so a new load of
                    // this name cannot reserve in between and get its state overwritten
                    let owned = {
                        let mut states = self_ref_clone.plugin_states.write().await;
                        let owned = {
                            let mut plugins = self_ref_clone
                                .plugins
                                .write()
                                .unwrap_or_else(std::sync::PoisonError::into_inner);
                            let index = plugins.iter().position(|p| p.token == token);
                            if let Some(index) = index {
                                plugins.remove(index);
                            }
                            index.is_some()
                        };
                        if owned {
                            states.insert(
                                plugin_name.clone(),
                                PluginState::Failed(error_msg.clone()),
                            );
                        }
                        owned
                    };

                    // Unloaded during on_unload
                    if !owned {
                        error!("Failed to initialize plugin {plugin_name}: {error_msg}");
                        state_notify.notify_waiters();
                        return Err(e);
                    }

                    // Keep it listed with its error, so it can be tried again
                    self_ref_clone.mark_inactive(
                        &plugin_path,
                        Some(&metadata),
                        PluginStatus::Failed(error_msg.clone()),
                    );
                    state_notify.notify_waiters();

                    error!("Failed to initialize plugin {plugin_name}: {error_msg}");
                    Err(e)
                }
            }
        });

        Ok((token, task))
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
        let loaders = self.get_loaders().await;

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

            // Loading a resident file again would build a second instance and loader handle
            if self.is_path_resident(&path) {
                continue;
            }

            // Find a loader that can handle this file
            let mut loader_found = false;
            for loader in &loaders {
                if loader.can_load(&path) {
                    match loader.load(&path).await {
                        Ok((instance, metadata, loader_data)) => {
                            // A rescan must not register a second copy of something still resident
                            if self.is_plugin_loaded(&metadata.name) {
                                // The instance's code lives in the library, so it goes first
                                drop(instance);
                                if loader.can_unload() {
                                    loader.unload(loader_data).await.ok();
                                } else {
                                    // Dropping the library here could unmap code that is still running
                                    Box::leak(loader_data);
                                }
                                loader_found = true;
                                break;
                            }

                            let plugin_override =
                                server.advanced_config.plugins.overrides.get(&metadata.name);

                            if plugin_override.is_some_and(|o| !o.enabled) {
                                info!(
                                    "Plugin \"{}\" is disabled in configuration, skipping.",
                                    metadata.name
                                );
                                self.mark_inactive(&path, Some(&metadata), PluginStatus::Disabled);
                                discard_loaded(instance, &**loader, loader_data).await;
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
                                self.mark_inactive(&path, Some(&metadata), PluginStatus::Unsigned);
                                discard_loaded(instance, &**loader, loader_data).await;
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
                        Err(err) => {
                            error!("Failed to load plugin from {:?}: {}", path, err);
                            self.mark_load_failed(&path, &err);
                            loader_found = true;
                        }
                    }
                    break;
                }
            }

            if !loader_found {
                self.unloaded_files.write().await.insert(path.clone());
            }
        }

        // Resolve dependencies
        let mut metadata_list: Vec<(String, Vec<String>)> = prepared_plugins
            .iter()
            .map(|(_, m, _, _, _)| (m.name.clone(), m.dependencies.clone()))
            .collect();
        // A needed dependency that is still loading must finish (or fail) before it counts. The
        // plugin calling this from its own on_load is never waited for, it would wait on itself
        let caller = self.initializing_caller();
        let needed: HashSet<&String> = metadata_list.iter().flat_map(|(_, deps)| deps).collect();
        let loading: Vec<String> = self
            .plugin_states
            .read()
            .await
            .iter()
            .filter(|(name, state)| {
                matches!(state, PluginState::Loading)
                    && needed.contains(name)
                    && caller.as_ref() != Some(*name)
            })
            .map(|(name, _)| name.clone())
            .collect();
        for name in loading {
            self.wait_for_initialization(&name).await?;
        }
        // Active plugins satisfy dependencies, they are already initialized, so they have none left
        // to resolve. Records still loading or kept after an unload are not active
        let resident: Vec<(String, Vec<String>)> = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|p| p.is_active)
            .map(|p| (p.metadata.name.clone(), Vec::new()))
            .collect();
        metadata_list.extend(resident);
        // A dependency that is neither prepared nor active fails only its dependents, in the
        // activity check below, not the whole pass
        let known: HashSet<String> = metadata_list.iter().map(|(name, _)| name.clone()).collect();
        for (_, deps) in &mut metadata_list {
            deps.retain(|dep| known.contains(dep));
        }

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
                // A dependency that failed or was skipped earlier in this pass is not active
                let missing = metadata
                    .dependencies
                    .iter()
                    .find(|dep| !self.is_plugin_active(dep));
                if let Some(dep) = missing {
                    let reason = format!("dependency {dep} is not active");
                    warn!("Skipping plugin \"{}\": {reason}", metadata.name);
                    self.mark_inactive(&path, Some(&metadata), PluginStatus::Failed(reason));
                    discard_loaded(instance, &*loader, loader_data).await;
                    continue;
                }

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
                    self.mark_inactive(&path, Some(&metadata), PluginStatus::PermissionDenied);
                    discard_loaded(instance, &*loader, loader_data).await;
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
                    Ok((_, task)) => {
                        // Its on_load waiting on this caller is refused instead of deadlocking
                        let _wait = self.enter_init_wait(&name)?;
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
    ) -> Result<(PluginId, tokio::task::JoinHandle<Result<(), String>>), ManagerError> {
        if !server.advanced_config.plugins.enabled {
            return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                "Plugin system is disabled in configuration".to_string(),
            )));
        }

        let loaders = self.get_loaders().await;
        for loader in &loaders {
            if loader.can_load(path) {
                // A loader took it, so it is no longer waiting for one
                self.unloaded_files.write().await.remove(path);

                let (instance, metadata, loader_data) = match loader.load(path).await {
                    Ok(loaded) => loaded,
                    Err(err) => {
                        error!("Failed to load plugin from {:?}: {}", path, err);
                        self.mark_load_failed(path, &err);
                        return Err(err.into());
                    }
                };

                let plugin_override = server.advanced_config.plugins.overrides.get(&metadata.name);

                if plugin_override.is_some_and(|o| !o.enabled) {
                    self.mark_inactive(path, Some(&metadata), PluginStatus::Disabled);
                    discard_loaded(instance, &**loader, loader_data).await;
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        format!("Plugin \"{}\" is disabled in configuration", metadata.name),
                    )));
                }

                let allow_unsigned = plugin_override
                    .and_then(|o| o.allow_unsigned)
                    .unwrap_or(server.advanced_config.plugins.allow_unsigned);

                if !allow_unsigned && !loader.is_signed(path) {
                    self.mark_inactive(path, Some(&metadata), PluginStatus::Unsigned);
                    discard_loaded(instance, &**loader, loader_data).await;
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        format!(
                            "Plugin \"{}\" is unsigned or invalid and allow_unsigned is disabled",
                            metadata.name
                        ),
                    )));
                }

                // A dependency still initializing must finish first, one that can't be waited on
                // counts as missing
                let mut missing = None;
                for dep in &metadata.dependencies {
                    if self.wait_for_initialization(dep).await.is_err()
                        || !self.is_plugin_active(dep)
                    {
                        missing = Some(dep.clone());
                        break;
                    }
                }
                if let Some(dep) = missing {
                    let reason = format!("dependency {dep} is not active");
                    self.mark_inactive(path, Some(&metadata), PluginStatus::Failed(reason.clone()));
                    discard_loaded(instance, &**loader, loader_data).await;
                    return Err(ManagerError::DependencyError(format!(
                        "Plugin \"{}\": {reason}",
                        metadata.name
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
                    self.mark_inactive(path, Some(&metadata), PluginStatus::PermissionDenied);
                    discard_loaded(instance, &**loader, loader_data).await;
                    return Err(ManagerError::LoaderError(LoaderError::RuntimeError(
                        "Permission denied".to_string(),
                    )));
                }

                let name = metadata.name.clone();
                let (token, task) = self
                    .spawn_plugin_initialization(
                        server.clone(),
                        instance,
                        metadata,
                        loader_data,
                        loader.clone(),
                        path.to_path_buf(),
                    )
                    .await?;
                return Ok((PluginId { name, token }, task));
            }
        }

        // No loader could handle this file, track it for future attempts
        self.unloaded_files.write().await.insert(path.to_path_buf());

        Err(ManagerError::PluginNotFound(
            path.to_string_lossy().to_string(),
        ))
    }

    /// Attempt to load a single plugin file, returns the plugin the file declares
    pub async fn try_load_plugin(
        self: &Arc<Self>,
        server: &Arc<Server>,
        path: &Path,
    ) -> Result<PluginId, ManagerError> {
        let (id, task) = self.start_loading_plugin(server, path).await?;
        // Its on_load waiting on this caller is refused instead of deadlocking
        let _wait = self.enter_init_wait(&id.name)?;
        task.await
            .map_err(|e| {
                ManagerError::LoaderError(LoaderError::InitializationFailed(format!(
                    "Task join error: {e}"
                )))
            })?
            .map_err(|e| ManagerError::LoaderError(LoaderError::InitializationFailed(e)))?;
        Ok(id)
    }

    /// Wait until a plugin is no longer initializing, whether it succeeded or failed.
    ///
    /// From an `on_load`, waiting on a plugin whose initialization waits on the caller (or on the
    /// caller itself) fails instead of deadlocking.
    pub async fn wait_for_initialization(&self, plugin_name: &str) -> Result<(), ManagerError> {
        let _wait = self.enter_init_wait(plugin_name)?;
        loop {
            // Create the waiter before checking, so a notify in between is not missed
            let notified = self.state_notify.notified();
            if !matches!(
                self.plugin_states.read().await.get(plugin_name),
                Some(PluginState::Loading)
            ) {
                return Ok(());
            }
            notified.await;
        }
    }

    /// The plugin of this manager whose `on_load` runs on this task, if any.
    fn initializing_caller(&self) -> Option<String> {
        INITIALIZING_PLUGIN
            .try_with(|(manager, name)| (*manager == self.id).then(|| name.clone()))
            .ok()
            .flatten()
    }

    /// Records that the calling `on_load` waits on `plugin_name`, unless that closes a cycle.
    ///
    /// `None` outside an `on_load`, nothing waits on such a caller.
    fn enter_init_wait(&self, plugin_name: &str) -> Result<Option<InitWait<'_>>, ManagerError> {
        let Ok(caller) = INITIALIZING_PLUGIN.try_with(Clone::clone) else {
            return Ok(None);
        };
        let awaited = (self.id, plugin_name.to_string());
        let mut waits = self
            .init_waits
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // Follow what `plugin_name` waits on, reaching the caller means it waits on the caller
        let mut pending = vec![awaited.clone()];
        let mut seen = HashSet::new();
        while let Some(name) = pending.pop() {
            if name == caller {
                return Err(ManagerError::InitializationDeadlock(
                    plugin_name.to_string(),
                ));
            }
            pending.extend(
                waits
                    .iter()
                    .filter(|(waiter, _)| *waiter == name)
                    .map(|(_, next)| next.clone()),
            );
            seen.insert(name);
            pending.retain(|name| !seen.contains(name));
        }
        let edge = (caller, awaited);
        waits.push(edge.clone());
        Ok(Some(InitWait {
            waits: &self.init_waits,
            edge,
        }))
    }

    /// Wait for a plugin to finish loading
    pub async fn wait_for_plugin(&self, plugin_name: &str) -> Result<(), ManagerError> {
        loop {
            self.wait_for_initialization(plugin_name).await?;
            match self.plugin_states.read().await.get(plugin_name).cloned() {
                Some(PluginState::Loaded) => return Ok(()),
                Some(PluginState::Failed(error)) => {
                    return Err(ManagerError::LoaderError(
                        LoaderError::InitializationFailed(error),
                    ));
                }
                // A new load reserved the name after the wait
                Some(PluginState::Loading) => {}
                None => return Err(ManagerError::PluginNotFound(plugin_name.to_string())),
            }
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

    /// Whether the record reserved under this token is still there
    fn holds_token(&self, token: u64) -> bool {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins.iter().any(|p| p.token == token)
    }

    /// Whether a loaded plugin came from this file.
    fn is_path_resident(&self, path: &Path) -> bool {
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        plugins.iter().any(|p| p.path == path)
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

    /// Every plugin file the manager knows about: running, unloaded, failed or without a loader.
    ///
    /// Sorted by name; the first, richer record wins when a file is listed more than once.
    pub async fn plugin_entries(&self) -> Vec<PluginEntry> {
        let states = self.plugin_states.read().await;

        let mut entries: Vec<PluginEntry> = {
            let plugins = self
                .plugins
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plugins
                .iter()
                .map(|p| PluginEntry {
                    status: if p.is_active && p.instance.is_some() {
                        PluginStatus::Active
                    } else {
                        match states.get(&p.metadata.name) {
                            Some(PluginState::Loading) => PluginStatus::Loading,
                            Some(PluginState::Failed(error)) => PluginStatus::Failed(error.clone()),
                            _ => PluginStatus::Unloaded,
                        }
                    },
                    metadata: Some(p.metadata.clone()),
                    path: p.path.clone(),
                    can_unload: p.loader.can_unload(),
                })
                .collect()
        };
        drop(states);

        {
            let inactive = self
                .inactive
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for entry in inactive.values() {
                push_unless_known(&mut entries, entry.clone());
            }
        }

        for path in self.unloaded_files.read().await.iter() {
            // Only files that claim to be a plugin, not a readme or a config next to them
            if !is_plugin_file(path) {
                continue;
            }
            push_unless_known(
                &mut entries,
                PluginEntry {
                    metadata: None,
                    path: path.clone(),
                    status: PluginStatus::NoLoader,
                    can_unload: true,
                },
            );
        }

        entries.sort_by(|a, b| a.name().cmp(&b.name()));
        entries
    }

    /// The plugin with this name, running or not.
    ///
    /// Reloading needs it: `unload_plugin` takes a name but `try_load_plugin` takes a file.
    pub async fn plugin_entry(&self, name: &str) -> Option<PluginEntry> {
        self.plugin_entries()
            .await
            .into_iter()
            .find(|entry| entry.name() == name)
    }

    /// Record a failed load, keeping the metadata an earlier record of the file already had.
    fn mark_load_failed(&self, path: &Path, err: &LoaderError) {
        let metadata = self
            .inactive
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(path)
            .and_then(|entry| entry.metadata.clone());
        self.mark_inactive(
            path,
            metadata.as_ref(),
            PluginStatus::Failed(err.to_string()),
        );
    }

    /// Records why a plugin file is not running, so it can be reported and not only logged.
    fn mark_inactive(&self, path: &Path, metadata: Option<&PluginMetadata>, status: PluginStatus) {
        self.inactive
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                path.to_path_buf(),
                PluginEntry {
                    metadata: metadata.cloned(),
                    path: path.to_path_buf(),
                    status,
                    // Nothing is resident, so there is nothing that could block a load
                    can_unload: true,
                },
            );
    }

    /// Names of every plugin the manager knows, running or not, for command completion.
    #[must_use]
    pub fn plugin_names(&self) -> Vec<String> {
        let mut names: Vec<String> = {
            let plugins = self
                .plugins
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            plugins.iter().map(|p| p.metadata.name.clone()).collect()
        };

        let inactive_names: Vec<String> = {
            let inactive = self
                .inactive
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inactive
                .values()
                .map(|entry| entry.name().into_owned())
                .collect()
        };
        names.extend(inactive_names);

        // Files without a loader are listed by file name, skip the lock if it is busy
        if let Ok(files) = self.unloaded_files.try_read() {
            names.extend(
                files
                    .iter()
                    .filter(|path| is_plugin_file(path))
                    .filter_map(|path| path.file_name())
                    .map(|name| name.to_string_lossy().into_owned()),
            );
        }

        names.sort();
        names.dedup();
        names
    }

    /// Names of plugins that are known but not running, for completing `load`.
    #[must_use]
    pub fn inactive_names(&self) -> Vec<String> {
        let mut names: Vec<String> = {
            let inactive = self
                .inactive
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            inactive
                .values()
                .map(|entry| entry.name().into_owned())
                .collect()
        };

        names.sort();
        names
    }

    /// Unload a plugin by name
    pub async fn unload_plugin(&self, name: &str) -> Result<(), ManagerError> {
        self.unload(name, None).await
    }

    /// Unload the plugin `id` names, not a later load of the same name.
    pub async fn unload_loaded(&self, id: &PluginId) -> Result<(), ManagerError> {
        self.unload(&id.name, Some(id.token)).await
    }

    /// `load`: only that load of `name`.
    async fn unload(&self, name: &str, load: Option<u64>) -> Result<(), ManagerError> {
        // The record is claimed, not removed: it keeps the name until the release attempt, so a
        // load of the same plugin is refused instead of running next to the old copy. State goes in
        // the same section, or a load reserving the name would lose its Loading state to this unload
        let (token, instance, context, loader, metadata, path) = loop {
            // on_load runs inside the library, so it must finish before the library is released
            self.wait_for_initialization(name).await?;

            let mut states = self.plugin_states.write().await;
            // A load can reserve the name between the wait and the lock
            if matches!(states.get(name), Some(PluginState::Loading)) {
                continue;
            }
            let mut plugins = self
                .plugins
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            // An inactive record is already unloaded, or another unload is releasing it
            let plugin = plugins
                .iter_mut()
                .find(|p| {
                    p.metadata.name == name
                        && p.is_active
                        && load.is_none_or(|token| p.token == token)
                })
                .ok_or_else(|| ManagerError::PluginNotFound(name.to_string()))?;
            plugin.is_active = false;
            states.remove(name);
            break (
                plugin.token,
                plugin.instance.take(),
                Arc::clone(&plugin.context),
                Arc::clone(&plugin.loader),
                plugin.metadata.clone(),
                plugin.path.clone(),
            );
        };

        self.unload_children(name).await;
        context.close();
        let owner = self.loader_owner(&loader).await;
        let mut registrations = self.take_registrations(name).await;
        context.unregister_commands();
        context.unregister_permissions();

        if let Some(instance) = &instance {
            match AssertUnwindSafe(async { instance.on_unload(context.clone()).await })
                .catch_unwind()
                .await
            {
                Ok(Ok(())) => {}
                Ok(Err(e)) => warn!("Plugin {name} failed to unload cleanly: {e}"),
                Err(_) => error!("Plugin {name} panicked in on_unload"),
            }
            // on_unload can register again, that must not outlive the library
            registrations.extend(self.take_registrations(name).await);
            context.unregister_commands();
            context.unregister_permissions();
        }
        self.pin_owner(&context, owner.as_deref());

        // The loader keeps the library, so the record stays as an inactive copy until restart
        if !loader.can_unload() {
            return Ok(());
        }

        let loader_data = self
            .plugins
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter_mut()
            .find(|p| p.token == token)
            .and_then(|p| p.loader_data.take());
        let unloaded = match loader_data {
            Some(data) => {
                let commands = context.has_commands();
                release_library(name, registrations, instance, &*loader, data, commands).await
            }
            None => Ok(()),
        };

        // Listed before the record goes, so a load that reserves the name next replaces the entry
        // instead of having it written over its own
        let status = match &unloaded {
            Ok(()) => PluginStatus::Unloaded,
            Err(e) => PluginStatus::Failed(e.to_string()),
        };
        self.mark_inactive(&path, Some(&metadata), status);
        // Also on `LibraryInUse`: the leaked library stays mapped, a new load may start
        self.plugins
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|p| p.token != token);

        unloaded.map_err(ManagerError::from)
    }

    /// The plugin that registered `loader`, if a plugin did.
    async fn loader_owner(&self, loader: &Arc<dyn PluginLoader>) -> Option<String> {
        self.loaders
            .read()
            .await
            .iter()
            .find(|registered| Arc::ptr_eq(&registered.loader, loader))
            .and_then(|registered| registered.owner.clone())
    }

    /// Commands of a plugin from another plugin's loader run that plugin's code too.
    /// Called after `on_unload`, which can still register commands.
    fn pin_owner(&self, context: &Context, owner: Option<&str>) {
        let Some(owner) = owner.filter(|_| context.has_commands()) else {
            return;
        };
        let plugins = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for plugin in plugins.iter().filter(|p| p.metadata.name == owner) {
            plugin.context.mark_commands();
        }
    }

    /// Unloads what `owner`'s loaders loaded: it can't run without them.
    async fn unload_children(&self, owner: &str) {
        let owned: Vec<Arc<dyn PluginLoader>> = self
            .loaders
            .read()
            .await
            .iter()
            .filter(|registered| registered.owner.as_deref() == Some(owner))
            .map(|registered| Arc::clone(&registered.loader))
            .collect();
        if owned.is_empty() {
            return;
        }
        let children: Vec<String> = self
            .plugins
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|p| owned.iter().any(|loader| Arc::ptr_eq(loader, &p.loader)))
            .map(|p| p.metadata.name.clone())
            .collect();
        for child in children {
            match Box::pin(self.unload_plugin(&child)).await {
                Ok(()) | Err(ManagerError::PluginNotFound(_)) => {}
                Err(e) => warn!("Plugin {child} loaded by {owner}: {e}"),
            }
        }
    }

    /// Takes a plugin's handlers, services and loaders off and returns them.
    async fn take_registrations(&self, source: &str) -> Registrations {
        let services = self.services.write().await.take_owned(source);
        let mut loaders = Vec::new();
        self.loaders.write().await.retain(|registered| {
            let keep = registered.owner.as_deref() != Some(source);
            if !keep {
                loaders.push(Arc::clone(&registered.loader));
            }
            keep
        });
        Registrations {
            handlers: self.unregister_handlers(source),
            services,
            loaders,
        }
    }

    /// Takes a plugin's handlers off the map and returns them.
    fn unregister_handlers(&self, source: &str) -> Vec<Arc<dyn DynEventHandler>> {
        let mut removed = Vec::new();
        self.handlers.rcu(|handlers| {
            // rcu retries on a concurrent change, only the last attempt counts
            removed.clear();
            let mut new_handlers = (**handlers).clone();
            new_handlers.retain(|_, handlers| {
                handlers.retain(|handler| {
                    let keep = handler.source() != Some(source);
                    if !keep {
                        removed.push(Arc::clone(handler));
                    }
                    keep
                });
                !handlers.is_empty()
            });
            Arc::new(new_handlers)
        });
        removed
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

    /// Wait for all plugins to finish loading.
    ///
    /// Fails from an `on_load`, the caller itself is still loading.
    pub async fn wait_for_all_plugins(&self) -> Result<(), ManagerError> {
        loop {
            let loading = self.get_loading_plugins().await;
            if loading.is_empty() {
                return Ok(());
            }
            // The caller is still loading, so waiting for everything would wait on itself
            if let Some(caller) = self.initializing_caller()
                && loading.contains(&caller)
            {
                return Err(ManagerError::InitializationDeadlock(caller));
            }
            for name in loading {
                self.wait_for_initialization(&name).await?;
            }
        }
    }

    /// Register an event handler
    pub fn register<E, H>(&self, handler: Arc<H>, priority: EventPriority, blocking: bool)
    where
        E: Payload + Send + Sync + 'static,
        H: EventHandler<E> + 'static,
    {
        let typed_handler = Arc::new(TypedEventHandler {
            handler,
            priority,
            blocking,
            source: None,
            _phantom: std::marker::PhantomData,
        });

        self.handlers.rcu(|handlers| {
            let mut new_handlers = (**handlers).clone();
            new_handlers
                .entry(E::get_name_static())
                .or_default()
                .push(typed_handler.clone());
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

    /// Fire an event to all registered handlers
    pub async fn fire<E: Payload + Send + Sync + 'static>(
        &self,
        server: &Arc<Server>,
        event: &mut E,
    ) {
        self.fire_dyn(server, E::get_name_static(), event).await;
    }

    async fn fire_dyn(
        &self,
        server: &Arc<Server>,
        event_name: &'static str,
        event: &mut (dyn Payload + Send + Sync),
    ) {
        let handlers_map = self.handlers.load();
        if handlers_map.is_empty() {
            return;
        }

        let Some(handlers) = handlers_map.get(event_name) else {
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

        // Run non-blocking handlers concurrently
        let non_blocking: Vec<_> = handlers
            .iter()
            .filter(|h| !h.is_blocking())
            .map(|h| h.handle_dyn(server, event))
            .collect();

        if !non_blocking.is_empty() {
            futures::future::join_all(non_blocking).await;
        }
    }

    /// Fire an event to all registered handlers synchronously (blocking if handlers exist).
    /// If no handlers are registered for this event, returns immediately without runtime overhead.
    pub fn fire_blocking<E: Payload + Send + Sync + 'static>(
        &self,
        server: &Arc<Server>,
        event: &mut E,
    ) {
        self.fire_blocking_dyn(server, E::get_name_static(), event);
    }

    fn fire_blocking_dyn(
        &self,
        server: &Arc<Server>,
        event_name: &'static str,
        event: &mut (dyn Payload + Send + Sync),
    ) {
        let handlers_map = self.handlers.load();
        if handlers_map.is_empty() {
            return;
        }

        let Some(handlers) = handlers_map.get(event_name) else {
            return;
        };

        if handlers.is_empty() {
            return;
        }

        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| {
                server
                    .runtime
                    .block_on(self.fire_dyn(server, event_name, event));
            });
        } else {
            server
                .runtime
                .block_on(self.fire_dyn(server, event_name, event));
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

    #[tokio::test]
    async fn same_named_plugin_of_another_manager_is_not_a_deadlock() {
        let manager = Arc::new(PluginManager::new(Vec::new()));
        let other = PluginManager::new(Vec::new());
        manager
            .plugin_states
            .write()
            .await
            .insert("a".to_string(), PluginState::Loading);

        // other's "a" loading does not make waiting on this manager's "a" a self-wait
        let result = INITIALIZING_PLUGIN
            .scope((other.id, "a".to_string()), async {
                let waiting = Arc::clone(&manager);
                let wait = tokio::spawn(async move { waiting.wait_for_all_plugins().await });
                tokio::task::yield_now().await;
                let finished = wait.is_finished();
                manager
                    .plugin_states
                    .write()
                    .await
                    .insert("a".to_string(), PluginState::Loaded);
                manager.state_notify.notify_waiters();
                (finished, wait.await)
            })
            .await;
        assert!(!result.0);
        assert!(matches!(result.1, Ok(Ok(()))));
    }

    #[tokio::test]
    async fn on_load_wait_cycles_fail_instead_of_hanging() {
        let manager = Arc::new(PluginManager::new(Vec::new()));
        let mut states = manager.plugin_states.write().await;
        states.insert("a".to_string(), PluginState::Loading);
        states.insert("b".to_string(), PluginState::Loading);
        drop(states);

        // An on_load waiting on itself
        let own = INITIALIZING_PLUGIN
            .scope(
                (manager.id, "a".to_string()),
                manager.wait_for_initialization("a"),
            )
            .await;
        assert!(matches!(own, Err(ManagerError::InitializationDeadlock(_))));

        // b's on_load waits on a, so a's on_load must not wait on b
        let waiting = Arc::clone(&manager);
        let b_waits_on_a = tokio::spawn(
            INITIALIZING_PLUGIN.scope((manager.id, "b".to_string()), async move {
                waiting.wait_for_initialization("a").await
            }),
        );
        while manager
            .init_waits
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_empty()
        {
            tokio::task::yield_now().await;
        }
        let mutual = INITIALIZING_PLUGIN
            .scope(
                (manager.id, "a".to_string()),
                manager.wait_for_initialization("b"),
            )
            .await;
        assert!(matches!(
            mutual,
            Err(ManagerError::InitializationDeadlock(_))
        ));

        // Once a finishes, b's wait ends and leaves no edge behind
        manager
            .plugin_states
            .write()
            .await
            .insert("a".to_string(), PluginState::Loaded);
        manager.state_notify.notify_waiters();
        assert!(matches!(b_waits_on_a.await, Ok(Ok(()))));
        assert!(
            manager
                .init_waits
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .is_empty()
        );
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
