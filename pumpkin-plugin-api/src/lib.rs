use crate::{events::EVENT_HANDLERS, logging::WitSubscriber};

pub mod events;

pub use wit::pumpkin::plugin::{
    command::{CommandSender, ConsumedArgs},
    context::{Context, Server},
};

pub mod logging;
pub mod text_component;

mod wit {
    wit_bindgen::generate!({
        skip: ["init-plugin"],
        path: "../pumpkin-plugin-wit/v0.1.0",
        world: "plugin",
    });

    use super::Component;
    export!(Component);
}

#[cfg(target_arch = "wasm32")]
#[unsafe(link_section = "pumpkin:api-version")]
#[used]
static API_VERSION: [u8; 5] = *b"0.1.0";

struct Component;
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
}

impl wit::exports::pumpkin::plugin::metadata::Guest for Component {
    fn get_metadata() -> wit::exports::pumpkin::plugin::metadata::PluginMetadata {
        let metadata = plugin().metadata();
        wit::exports::pumpkin::plugin::metadata::PluginMetadata {
            name: metadata.name,
            version: metadata.version,
            authors: metadata.authors,
            description: metadata.description,
        }
    }
}

impl wit::Guest for Component {
    fn on_load(context: Context) -> Result<(), String> {
        plugin().on_load(context)
    }

    fn on_unload(context: Context) -> Result<(), String> {
        plugin().on_unload(context)
    }

    fn handle_event(event_id: u32, server: Server, event: events::Event) -> events::Event {
        let handlers = EVENT_HANDLERS.lock().unwrap();
        if let Some(handler) = handlers.get(&event_id) {
            handler.handle_erased(server, event)
        } else {
            event
        }
    }

    fn handle_command(command_id: u32, sender: CommandSender, server: Server, args: ConsumedArgs) {
        todo!()
    }
}

pub type Result<T, E = String> = core::result::Result<T, E>;

/// The trait that every Pumpkin plugin must implement.
pub trait Plugin: Send + Sync {
    /// Create a new instance of the plugin.
    fn new() -> Self
    where
        Self: Sized;

    /// Define the metadata for the plugin.
    fn metadata(&self) -> PluginMetadata;

    /// Called when the plugin is loaded by the server.
    fn on_load(&mut self, _context: Context) -> Result<()> {
        Ok(())
    }

    /// Called when the plugin is unloaded by the server.
    fn on_unload(&mut self, _context: Context) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
pub fn register_plugin(build_plugin: fn() -> Box<dyn Plugin>) {
    let _ = tracing::subscriber::set_global_default(WitSubscriber::new());
    unsafe { PLUGIN = Some((build_plugin)()) }
}

fn plugin() -> &'static mut dyn Plugin {
    #[expect(static_mut_refs)]
    unsafe {
        PLUGIN.as_deref_mut().unwrap()
    }
}

static mut PLUGIN: Option<Box<dyn Plugin>> = None;

/// Registers the provided type as a Pumpkin plugin.
///
/// The type must implement the [`Plugin`] trait.
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        #[unsafe(export_name = "init-plugin")]
        pub extern "C" fn __init_plugin() {
            $crate::register_plugin(|| Box::new(<$plugin_type as $crate::Plugin>::new()));
        }
    };
}
