pub mod api;

pub use api::*;
use async_trait::async_trait;
use std::{collections::HashMap, fs, path::Path, sync::Arc};
use tokio::sync::RwLock;

use crate::server::Server;

type PluginData = (
    PluginMetadata<'static>,
    Box<dyn Plugin>,
    libloading::Library,
    bool,
);

#[async_trait]
pub trait DynEventHandler: Send + Sync {
    async fn handle_dyn(&self, event: &(dyn Event + Send + Sync));
    async fn handle_blocking_dyn(&self, _event: &mut (dyn Event + Send + Sync));
    fn is_blocking(&self) -> bool;
    fn get_priority(&self) -> EventPriority;
}

#[async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    async fn handle(&self, _event: &E) {
        unimplemented!();
    }
    async fn handle_blocking(&self, _event: &mut E) {
        unimplemented!();
    }
}

struct TypedEventHandler<E, H>
where
    E: Event + Send + Sync + 'static,
    H: EventHandler<E> + Send + Sync,
{
    handler: H,
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
    async fn handle_blocking_dyn(&self, event: &mut (dyn Event + Send + Sync)) {
        // Check if the event is the same type as E. We can not use the type_id because it is
        // different in the plugin and the main program
        if E::get_name_static() == event.get_name() {
            // This is fully safe as long as the event's get_name() and get_name_static()
            // functions are correctly implemented and don't conflict with other events
            let event = unsafe {
                &mut *std::ptr::from_mut::<dyn std::any::Any>(event.as_any_mut()).cast::<E>()
            };
            self.handler.handle_blocking(event).await;
        }
    }

    async fn handle_dyn(&self, event: &(dyn Event + Send + Sync)) {
        // Check if the event is the same type as E. We can not use the type_id because it is
        // different in the plugin and the main program
        if E::get_name_static() == event.get_name() {
            // This is fully safe as long as the event's get_name() and get_name_static()
            // functions are correctly implemented and don't conflict with other events
            let event =
                unsafe { &*std::ptr::from_ref::<dyn std::any::Any>(event.as_any()).cast::<E>() };
            self.handler.handle(event).await;
        }
    }

    fn is_blocking(&self) -> bool {
        self.blocking
    }

    fn get_priority(&self) -> EventPriority {
        self.priority.clone()
    }
}

pub type HandlerMap = HashMap<&'static str, Vec<Box<dyn DynEventHandler>>>;

pub struct PluginManager {
    plugins: Vec<PluginData>,
    server: Option<Arc<Server>>,
    handlers: Arc<RwLock<HandlerMap>>,
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
            handlers: Arc::new(RwLock::new(HashMap::new())),
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

        let dir_entries = fs::read_dir(PLUGIN_DIR);

        for entry in dir_entries.unwrap() {
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

    #[must_use]
    pub fn list_plugins(&self) -> Vec<(&PluginMetadata, &bool)> {
        self.plugins
            .iter()
            .map(|(metadata, _, _, loaded)| (metadata, loaded))
            .collect()
    }

    pub async fn register<E: Event + 'static, H>(
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

    pub async fn fire<E: Event + Send + Sync + 'static>(&self, mut event: E) -> E {
        // Take a snapshot of handlers to avoid lifetime issues
        let handlers = self.handlers.read().await;

        log::debug!("Firing event: {}", E::get_name_static());

        if let Some(handlers_vec) = handlers.get(&E::get_name_static()) {
            log::debug!(
                "Found {} handlers for event: {}",
                handlers_vec.len(),
                E::get_name_static()
            );

            let (blocking_handlers, non_blocking_handlers): (Vec<_>, Vec<_>) = handlers_vec
                .iter()
                .partition(|handler| handler.is_blocking());

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
