//! Pumpkin plugin API.
//!
//! This crate provides everything needed to write a Pumpkin server plugin compiled
//! to WebAssembly. A plugin consists of a type that implements [`Plugin`], registered
//! with the [`register_plugin!`] macro.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use pumpkin_plugin_api::{Plugin, PluginMetadata, Context, register_plugin, permissions::permissions};
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn new() -> Self { MyPlugin }
//!     fn metadata(&self) -> PluginMetadata {
//!         PluginMetadata {
//!             name: "my-plugin".into(),
//!             version: "0.1.0".into(),
//!             authors: vec!["you".into()],
//!             description: "An example plugin.".into(),
//!             dependencies: vec![],
//!             permissions: vec![permissions::NETWORK_DNS.into()],
//!         }
//!     }
//! }
//!
//! register_plugin!(MyPlugin);
//! ```

use crate::{
    commands::COMMAND_HANDLERS, events::EVENT_HANDLERS, logging::WitSubscriber,
    scheduler::TASK_HANDLERS, text::TextComponent,
};

pub mod commands;
pub mod events;
/// Constants for plugin permissions.
///
/// Use these in your `PluginMetadata` to request access to specific host features.
pub mod permissions;
pub mod scheduler;

pub mod command {
    pub use crate::wit::pumpkin::plugin::command::{
        Command, CommandError, CommandNode, CommandSender, ConsumedArgs,
    };
}

pub use wit::pumpkin::plugin::{
    bedrock_packets, block_entity, boss_bar, command as command_wit, common,
    context::{Context, Server},
    entity, entity_types, gui, i18n, java_packets, particles, permission, player, scoreboard,
    server, text, world,
};

pub mod logging;

#[allow(clippy::too_many_arguments)]
mod wit {
    wit_bindgen::generate!({
        skip: ["init-plugin"],
        path: "../pumpkin-plugin-wit/v0.1",
        world: "plugin",
        enable_method_chaining: true
    });

    use super::Component;
    export!(Component);
}

struct Component;

/// Metadata that describes a plugin to the server.
pub struct PluginMetadata {
    /// The human-readable name of the plugin.
    pub name: String,
    /// The plugin's version string (e.g. `"1.0.0"`).
    pub version: String,
    /// The list of plugin authors.
    pub authors: Vec<String>,
    /// A short description of what the plugin does.
    pub description: String,
    /// The list of plugin dependencies.
    pub dependencies: Vec<String>,
    /// The list of permissions requested by the plugin.
    pub permissions: Vec<String>,
}

impl wit::exports::pumpkin::plugin::metadata::Guest for Component {
    /// Returns the plugin metadata to the host.
    fn get_metadata() -> wit::exports::pumpkin::plugin::metadata::PluginMetadata {
        let metadata = plugin().metadata();
        wit::exports::pumpkin::plugin::metadata::PluginMetadata {
            name: metadata.name,
            version: metadata.version,
            authors: metadata.authors,
            description: metadata.description,
            dependencies: metadata.dependencies,
            permissions: metadata.permissions,
        }
    }
}

impl wit::Guest for Component {
    /// WIT entry point — delegates to [`Plugin::on_load`].
    fn on_load(context: Context) -> Result<(), String> {
        plugin().on_load(context)
    }

    /// WIT entry point — delegates to [`Plugin::on_unload`].
    fn on_unload(context: Context) -> Result<(), String> {
        plugin().on_unload(context)
    }

    /// WIT entry point — dispatches an incoming event to the registered handler for `event_id`.
    ///
    /// Returns the event unchanged if no handler is registered for the given id.
    fn handle_event(event_id: u32, server: Server, event: events::Event) -> events::Event {
        let handlers = EVENT_HANDLERS.lock().unwrap();
        if let Some(handler) = handlers.get(&event_id) {
            handler.handle_erased(server, event)
        } else {
            event
        }
    }

    /// WIT entry point — dispatches an incoming command invocation to the registered handler for `command_id`.
    ///
    /// Returns a [`CommandError`](command::CommandError) if no handler is registered for the given id.
    fn handle_command(
        command_id: u32,
        sender: command::CommandSender,
        server: Server,
        args: command::ConsumedArgs,
    ) -> Result<i32, command::CommandError> {
        let handlers = COMMAND_HANDLERS.lock().unwrap();
        if let Some(handler) = handlers.get(&command_id) {
            handler.handle(sender, server, args)
        } else {
            Err(command::CommandError::CommandFailed(TextComponent::text(
                &format!("no handler registered for command id {command_id}"),
            )))
        }
    }

    /// WIT entry point — dispatches a scheduled task invocation to the registered handler for `handler_id`.
    fn handle_task(handler_id: u32, server: Server) {
        let mut handlers = TASK_HANDLERS.lock().unwrap();
        handlers.handle(handler_id, server);
    }
}

/// Convenience alias for `core::result::Result<T, String>` used throughout the plugin API.
pub type Result<T, E = String> = core::result::Result<T, E>;

/// The trait that every Pumpkin plugin must implement.
///
/// Use the [`register_plugin!`] macro to register your implementation with the runtime.
pub trait Plugin: Send + Sync {
    /// Creates a new instance of the plugin.
    ///
    /// Called once by the runtime before [`on_load`](Plugin::on_load).
    fn new() -> Self
    where
        Self: Sized;

    /// Returns the metadata for this plugin.
    fn metadata(&self) -> PluginMetadata;

    /// Called when the plugin is loaded by the server.
    ///
    /// Use this to register event handlers, commands, and perform any setup work.
    fn on_load(&mut self, _context: Context) -> Result<()> {
        Ok(())
    }

    /// Called when the plugin is unloaded by the server.
    ///
    /// Use this to clean up any resources acquired during [`on_load`](Plugin::on_load).
    fn on_unload(&mut self, _context: Context) -> Result<()> {
        Ok(())
    }
}

#[doc(hidden)]
pub fn register_plugin(build_plugin: fn() -> Box<dyn Plugin>) {
    let _ = tracing::subscriber::set_global_default(WitSubscriber::new());
    unsafe { PLUGIN = Some(build_plugin()) }
}

/// Returns a mutable reference to the currently loaded plugin instance.
///
/// # Panics
/// If called before [`register_plugin`] has initialized `PLUGIN`.
fn plugin() -> &'static mut dyn Plugin {
    #[expect(static_mut_refs)]
    unsafe {
        PLUGIN.as_deref_mut().unwrap()
    }
}

/// The singleton plugin instance, initialised by [`register_plugin`].
static mut PLUGIN: Option<Box<dyn Plugin>> = None;

/// Registers the provided type as a Pumpkin plugin.
///
/// This macro generates the WebAssembly export entry point that the server uses to
/// instantiate the plugin. The type must implement the [`Plugin`] trait.
///
/// # Example
/// ```rust,ignore
/// register_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        #[unsafe(export_name = "init-plugin")]
        pub extern "C" fn __init_plugin() {
            $crate::register_plugin(|| Box::new(<$plugin_type as $crate::Plugin>::new()));
        }
    };
}
