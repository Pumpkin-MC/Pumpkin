use std::{fs, path::Path, sync::Arc};

use pumpkin_util::PermissionLvl;
use tokio::sync::RwLock;

use crate::{
    entity::player::Player,
    plugin::{EventHandler, HandlerMap, TypedEventHandler},
    server::Server,
};

use super::{Event, EventPriority, PluginMetadata};

pub struct Context {
    metadata: PluginMetadata<'static>,
    pub server: Arc<Server>,
    handlers: Arc<RwLock<HandlerMap>>,
}
impl Context {
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

    #[must_use]
    pub fn get_data_folder(&self) -> String {
        let path = format!("./plugins/{}", self.metadata.name);
        if !Path::new(&path).exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path
    }

    pub async fn get_player_by_name(&self, player_name: String) -> Option<Arc<Player>> {
        self.server.get_player_by_name(&player_name).await
    }

    pub async fn register_command(
        &self,
        tree: crate::command::tree::CommandTree,
        permission: PermissionLvl,
    ) {
        let mut dispatcher_lock = self.server.command_dispatcher.write().await;
        dispatcher_lock.register(tree, permission);
    }

    pub async fn register_event<E: Event + 'static, H>(
        &self,
        handler: H,
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
