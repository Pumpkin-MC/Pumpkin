use std::{fs, path::Path, sync::Arc};

use crate::command::client_suggestions;
use pumpkin_util::PermissionLvl;
use tokio::sync::RwLock;

use crate::{
    entity::player::Player,
    plugin::{EventHandler, HandlerMap, TypedEventHandler},
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
    handlers: Arc<RwLock<HandlerMap>>,
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
    ) -> Self {
        Self {
            metadata,
            server,
            handlers,
        }
    }

    /// Retrieves the data folder path for the plugin, creating it if it does not exist.
    ///
    /// # Returns
    /// A string representing the path to the data folder.
    #[must_use]
    pub fn get_data_folder(&self) -> String {
        let path = format!("./plugins/{}", self.metadata.name);
        if !Path::new(&path).exists() {
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

    /// Asynchronously registers a command with the server.
    ///
    /// # Arguments
    /// - `tree`: The command tree to register.
    /// - `permission`: The permission level required to execute the command.
    pub async fn register_command(
        &self,
        tree: crate::command::tree::CommandTree,
        permission: PermissionLvl,
    ) {
        {
            let mut dispatcher_lock = self.server.command_dispatcher.write().await;
            dispatcher_lock.register(tree, permission);
        };

        for world in self.server.worlds.read().await.iter() {
            for player in world.players.read().await.values() {
                client_suggestions::send_c_commands_packet(player, &self.server.command_dispatcher)
                    .await;
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
                client_suggestions::send_c_commands_packet(player, &self.server.command_dispatcher)
                    .await;
            }
        }
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
}
