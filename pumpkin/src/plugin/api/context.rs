use std::{any::Any, fs, path::Path, path::PathBuf, sync::Arc};

use crate::command::client_suggestions;
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionManager},
};
use tokio::sync::RwLock;

use crate::{
    entity::player::Player,
    plugin::{EventHandler, HandlerMap, PluginManager, TypedEventHandler},
    server::Server,
};

use super::{Event, EventPriority, PluginMetadata};

/// The `Context` struct represents the context of a plugin, containing metadata,
/// a server reference, and event handlers.
///
/// # Fields
/// - `metadata`: Metadata of the plugin.
/// - `server`: A reference to the server on which the plugin operates.
/// - `handlers`: A map of event handlers, protected by a read-write lock for safe access across threads.
pub struct Context {
    metadata: PluginMetadata<'static>,
    pub server: Arc<Server>,
    pub handlers: Arc<RwLock<HandlerMap>>,
    pub plugin_manager: Arc<PluginManager>,
    pub permission_manager: Arc<RwLock<PermissionManager>>,
}
impl Context {
    /// Creates a new instance of `Context`.
    ///
    /// # Arguments
    /// - `metadata`: The metadata of the plugin.
    /// - `server`: A reference to the server.
    /// - `handlers`: A collection containing the event handlers.
    ///
    /// # Returns
    /// A new instance of `Context`.
    #[must_use]
    pub fn new(
        metadata: PluginMetadata<'static>,
        server: Arc<Server>,
        handlers: Arc<RwLock<HandlerMap>>,
        plugin_manager: Arc<PluginManager>,
        permission_manager: Arc<RwLock<PermissionManager>>,
    ) -> Self {
        Self {
            metadata,
            server,
            handlers,
            plugin_manager,
            permission_manager,
        }
    }

    /// Retrieves the data folder path for the plugin, creating it if it does not exist.
    ///
    /// # Returns
    /// A string representing the path to the data folder.
    #[must_use]
    pub fn get_data_folder(&self) -> PathBuf {
        let path = Path::new("./plugins").join(self.metadata.name);
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path
    }

    /// Asynchronously retrieves a player by their name.
    ///
    /// # Arguments
    /// - `player_name`: The name of the player to retrieve.
    ///
    /// # Returns
    /// An optional reference to the player if found, or `None` if not.
    pub async fn get_player_by_name(&self, player_name: String) -> Option<Arc<Player>> {
        self.server.get_player_by_name(&player_name).await
    }

    /// Registers a service with the plugin context.
    ///
    /// This method allows you to associate a service instance with a given name,
    /// making it available for retrieval by plugins or other components.
    /// The service must be wrapped in an `Arc` and implement `Any`, `Send`, and `Sync`.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name to register the service under.
    /// * `service` - The service instance to register, wrapped in an `Arc`.
    ///
    /// # Example
    ///
    /// ```
    /// context.register_service("my_service".to_string(), Arc::new(MyService::new())).await;
    /// ```
    pub async fn register_service<T: Any + Send + Sync>(&self, name: String, service: Arc<T>) {
        let mut services = self.plugin_manager.services.write().await;
        services.insert(name, service);
    }

    /// Retrieves a registered service by name and type.
    ///
    /// This method attempts to fetch a service previously registered under the given name,
    /// and downcasts it to the requested type. Returns `Some(Arc<T>)` if the service exists
    /// and the type matches, or `None` otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the service to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option<Arc<T>>` containing the service if found and type matches, or `None`.
    ///
    /// # Example
    ///
    /// ```
    /// if let Some(service) = context.get_service::<MyService>("my_service").await {
    ///     // Use the service
    /// }
    /// ```
    pub async fn get_service<T: Any + Send + Sync>(&self, name: &str) -> Option<Arc<T>> {
        let services = self.plugin_manager.services.read().await;
        services.get(name)?.clone().downcast::<T>().ok()
    }

    /// Asynchronously registers a command with the server.
    ///
    /// # Arguments
    /// - `tree`: The command tree to register.
    /// - `permission`: The permission level required to execute the command.
    pub async fn register_command(
        &self,
        tree: crate::command::tree::CommandTree,
        permission_node: &str,
    ) {
        let plugin_name = self.metadata.name;
        let full_permission_node = if permission_node.contains(':') {
            permission_node.to_string()
        } else {
            format!("{plugin_name}:{permission_node}")
        };

        {
            let mut dispatcher_lock = self.server.command_dispatcher.write().await;
            dispatcher_lock.register(tree, full_permission_node.as_str());
        };

        for world in self.server.worlds.read().await.iter() {
            for player in world.players.read().await.values() {
                let command_dispatcher = self.server.command_dispatcher.read().await;
                client_suggestions::send_c_commands_packet(player, &command_dispatcher).await;
            }
        }
    }

    /// Asynchronously unregisters a command from the server.
    ///
    /// # Arguments
    /// - `name`: The name of the command to unregister.
    pub async fn unregister_command(&self, name: &str) {
        {
            let mut dispatcher_lock = self.server.command_dispatcher.write().await;
            dispatcher_lock.unregister(name);
        };

        for world in self.server.worlds.read().await.iter() {
            for player in world.players.read().await.values() {
                let command_dispatcher = self.server.command_dispatcher.read().await;
                client_suggestions::send_c_commands_packet(player, &command_dispatcher).await;
            }
        }
    }

    /// Register a permission for this plugin
    pub async fn register_permission(&self, permission: Permission) -> Result<(), String> {
        // Ensure the permission has the correct namespace
        let plugin_name = self.metadata.name;

        if !permission.node.starts_with(&format!("{plugin_name}:")) {
            return Err(format!(
                "Permission {} must use the plugin's namespace ({})",
                permission.node, plugin_name
            ));
        }

        let manager = self.permission_manager.read().await;
        let mut registry = manager.registry.write().await;
        registry.register_permission(permission)
    }

    /// Check if a player has a permission
    pub async fn player_has_permission(&self, player_uuid: &uuid::Uuid, permission: &str) -> bool {
        let permission_manager = self.permission_manager.read().await;

        // If the player isn't online, we need to find their op level
        let player_op_level = (self.server.get_player_by_uuid(*player_uuid).await)
            .map_or(PermissionLvl::Zero, |player| player.permission_lvl.load());

        permission_manager
            .has_permission(player_uuid, permission, player_op_level)
            .await
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
    pub async fn register_event<E: Event + 'static, H>(
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

    /// Registers a custom plugin loader that can load additional plugin types.
    ///
    /// This method allows plugins to extend the server with support for loading
    /// plugins in different formats (e.g., Lua, JavaScript, Python). When a new
    /// loader is registered, the plugin manager will automatically attempt to load
    /// any previously unloadable files in the plugins directory with this new loader.
    ///
    /// # Arguments
    /// - `loader`: The custom plugin loader implementation to register.
    ///
    /// # Returns
    /// `true` if new plugins were loaded as a result of registering this loader, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// // Create and register a custom Lua plugin loader
    /// let lua_loader = Arc::new(LuaPluginLoader::new());
    /// context.register_plugin_loader(lua_loader).await;
    /// ```
    pub async fn register_plugin_loader(
        &self,
        loader: Arc<dyn crate::plugin::loader::PluginLoader>,
    ) -> bool {
        let before_count = self.plugin_manager.loaded_plugins().await.len();
        self.plugin_manager.add_loader(loader).await;
        let after_count = self.plugin_manager.loaded_plugins().await.len();

        // Return true if any new plugins were loaded
        after_count > before_count
    }
}
