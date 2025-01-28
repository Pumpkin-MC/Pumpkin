pub mod api;

pub use api::*;
use std::{fs, path::Path, sync::Arc};

use crate::server::Server;

type PluginData = (
    PluginMetadata<'static>,
    Box<dyn Plugin>,
    libloading::Library,
    bool,
);

pub struct PluginManager {
    plugins: Vec<PluginData>,
    server: Option<Arc<Server>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            plugins: vec![],
            server: None,
        }
    }

    pub fn set_server(&mut self, server: Arc<Server>) {
        self.server = Some(server);
    }

    pub async fn load_plugins(&mut self) -> Result<(), String> {
        const PLUGIN_DIR: &str = "./plugins";

        if !Path::new(PLUGIN_DIR).exists() {
            fs::create_dir(PLUGIN_DIR).unwrap();
        }

        let dir_entires = fs::read_dir(PLUGIN_DIR);

        for entry in dir_entires.unwrap() {
            if !entry.as_ref().unwrap().path().is_file() {
                continue;
            }
            self.try_load_plugin(entry.unwrap().path().as_path()).await;
        }

        Ok(())
    }

    async fn try_load_plugin(&mut self, path: &Path) {
        let library = unsafe { libloading::Library::new(path).unwrap() };

        let plugin_fn = unsafe { library.get::<fn() -> Box<dyn Plugin>>(b"plugin").unwrap() };
        let metadata: &PluginMetadata =
            unsafe { &**library.get::<*const PluginMetadata>(b"METADATA").unwrap() };

        // The chance that this will panic is non-existent, but just in case
        let context = Context::new(
            metadata.clone(),
            self.server.clone().expect("Server not set"),
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
    }

    #[must_use]
    pub fn is_plugin_loaded(&self, name: &str) -> bool {
        self.plugins
            .iter()
            .any(|(metadata, _, _, loaded)| metadata.name == name && *loaded)
    }

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
            );
            let res = plugin.on_load(&context).await;
            res?;
            *loaded = true;
            Ok(())
        } else {
            Err(format!("Plugin {name} not found"))
        }
    }

    pub async fn unload_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|(metadata, _, _, _)| metadata.name == name);

        if let Some((metadata, plugin, _, loaded)) = plugin {
            let context = Context::new(
                metadata.clone(),
                self.server.clone().expect("Server not set"),
            );
            let res = plugin.on_unload(&context).await;
            res?;
            *loaded = false;
            Ok(())
        } else {
            Err(format!("Plugin {name} not found"))
        }
    }

    #[must_use]
    pub fn list_plugins(&self) -> Vec<(&PluginMetadata, &bool)> {
        self.plugins
            .iter()
            .map(|(metadata, _, _, loaded)| (metadata, loaded))
            .collect()
    }
}
