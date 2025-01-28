use std::{fs, path::Path, sync::Arc};

use pumpkin_util::PermissionLvl;

use crate::server::Server;

use super::PluginMetadata;

pub struct Context {
    metadata: PluginMetadata<'static>,
    pub server: Arc<Server>,
}
impl Context {
    #[must_use]
    pub fn new(metadata: PluginMetadata<'static>, server: Arc<Server>) -> Self {
        Self { metadata, server }
    }

    #[must_use]
    pub fn get_data_folder(&self) -> String {
        let path = format!("./plugins/{}", self.metadata.name);
        if !Path::new(&path).exists() {
            fs::create_dir_all(&path).unwrap();
        }
        path
    }

    pub async fn register_command(
        &self,
        tree: crate::command::tree::CommandTree,
        permission: PermissionLvl,
    ) {
        let mut dispatcher_lock = self.server.command_dispatcher.write().await;
        dispatcher_lock.register(tree, permission);
    }
}
