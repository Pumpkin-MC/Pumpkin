//! Pumpkin plugin API.
#![warn(missing_docs)]
#![allow(
    clippy::undocumented_unsafe_blocks,
    clippy::option_if_let_else,
    clippy::collection_is_never_read,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::panic
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
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
//!
//! # Persisting data
//!
//! Plugins run as WebAssembly with WASI, so a plugin stores data that survives
//! server restarts by reading and writing its own files with your language's
//! normal file API (for example `std::fs` in Rust). There is no separate
//! storage API; the file system is the storage.
//!
//! Each plugin has a private data folder. To use it:
//!
//! 1. Request the `fs.read.data` and/or `fs.write.data` permissions
//!    (`permissions::FS_READ_DATA` / `permissions::FS_WRITE_DATA`) in your
//!    [`PluginMetadata`]. Without them the folder is not accessible.
//! 2. Get the folder path from the context's `get_data_folder` method inside
//!    `on_load` or `on_unload`. The returned path is the folder as seen from
//!    inside the WASI sandbox.
//! 3. Read and write files under that path with your normal file API.
//!
//! ```rust,ignore
//! fn on_load(&self, context: &Context) -> Result<(), String> {
//!     let path = format!("{}/state.json", context.get_data_folder());
//!     let saved = std::fs::read_to_string(&path).unwrap_or_default();
//!     // ...parse and use `saved`, then later write it back...
//!     Ok(())
//! }
//! ```

use crate::{
    commands::{COMMAND_HANDLERS, COMMAND_SUGGESTION_HANDLERS},
    events::EVENT_HANDLERS,
    logging::WitSubscriber,
    scheduler::TASK_HANDLERS,
    text::TextComponent,
};

/// Plugin command registration and handling utilities.
pub mod commands;
/// Display and interaction entity utilities and builders.
pub mod display;
/// Custom enchantment registration and builder utilities.
pub mod enchantment;
/// Event system and event handlers.
pub mod events;
mod ext;
/// Bedrock UI form builders.
pub mod forms;
/// Constants for plugin permissions.
///
/// Use these in your `PluginMetadata` to request access to specific host features.
pub mod permissions;
/// Custom recipe registration and builder utilities.
pub mod recipe;
/// Scheduler utilities.
pub mod scheduler;
/// Scoreboard team management and builder utilities.
pub mod team;
/// Custom world and chunk generation utilities and traits.
pub mod worldgen;

/// Command WIT API re-exports.
pub mod command {
    pub use crate::wit::pumpkin::plugin::command::{
        Arg, ArgumentType, Command, CommandError, CommandNode, CommandSender, CommandSuggestion,
        CommandSuggestions, ConsumedArgs, StringType, SuggestionRequest,
    };
}

pub use wit::pumpkin::plugin::{
    advancement as advancement_wit, bedrock_packets, block_entity, boss_bar,
    command as command_wit, common,
    context::{self, Context, MarketplaceMetadata, Server},
    damage_types as damage_types_wit, data_components, display as display_wit,
    enchantments as enchantments_wit, entity,
    entity_types::EntityType,
    event::{self as events_wit, EventType},
    gui, i18n, ipc, item_stack, java_dialogs, java_packets, particles, permission, player,
    recipe as recipe_wit, scoreboard, screens as screens_wit, server, statistics as statistics_wit,
    text, uuid, world,
};

// Convenience re-exports of commonly-used plugin types so plugin authors can
// name them directly (e.g. build an `ItemStack` for a GUI or `/give`).
pub use damage_types_wit::DamageType;
pub use display::{
    BillboardMode, BlockDisplayEntity, DisplayEntity, DisplayEntityExt, DisplayTransformation,
    EntityDisplayExt, InteractionEntity, ItemDisplayEntity, ItemDisplayEntityExt, ItemDisplayMode,
    Quaternionf, TextAlignment, TextDisplayEntity, TextDisplayEntityExt, TransformationBuilder,
    Vector3f,
};
pub use enchantment::{
    AttributeModifierSlot, CustomEnchantment, CustomEnchantmentValue, Enchantment,
    EnchantmentBuilder, EnchantmentError, EnchantmentManager, RegistrableEnchantment,
};
pub use events::{EventHandler, FromIntoEvent};
pub use ext::player::PlayerEnderChestExt;
pub use recipe::{
    CookingRecipeBuilder, Ingredient, RecipeCategory, RecipeError, RecipeManager,
    RegistrableRecipe, ShapedRecipeBuilder, ShapelessRecipeBuilder,
};
pub use screens_wit::Screen;
pub use statistics_wit::{CustomStatistic, StatisticCategory};
pub use team::{PlayerTeamExt, ScoreboardTeamExt, Team, TeamSettingsBuilder};
pub use wit::pumpkin::plugin::item_stack::ItemStack;
pub use wit::pumpkin::plugin::player::Player;
pub use wit::pumpkin::plugin::scoreboard::{CollisionRule, NametagVisibility, TeamSettings};
pub use wit::pumpkin::plugin::server::Dimension;
pub use wit::pumpkin::plugin::world::{
    Block, BlockDirection, BlockState, BlockStateInfo, Entity, Flammable, RayTraceBlockResult,
    RayTraceEntityResult, RaycastResult, World, WorldBorder,
};
pub use worldgen::{ChunkBuffer, ChunkGenerator, GenerationPhase, GeneratorManager};

/// Advancement WIT API re-exports.
pub mod advancement {
    pub use crate::wit::pumpkin::plugin::advancement::{
        AdvancementDisplay, AdvancementInfo, AdvancementProgress, FrameType,
    };
}

/// Java dialog WIT API re-exports.
pub mod java_dialog {
    pub use crate::wit::pumpkin::plugin::java_dialogs::{
        Action, ActionButton, AfterAction, CustomClickAction, Dialog, DialogBody, DialogInput,
        DialogInputBool, DialogInputNumberRange, DialogInputSingleOption, DialogInputText,
        DialogType, Link, LinkLabel, LinkType,
    };
}

/// WIT-based logging subscriber.
pub mod logging;

#[allow(clippy::too_many_arguments, missing_docs)]
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
        let handlers = EVENT_HANDLERS.lock().unwrap_or_else(|e| e.into_inner());
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
        let handlers = COMMAND_HANDLERS.lock().unwrap_or_else(|e| e.into_inner());
        handlers.get(&command_id).map_or_else(
            || {
                Err(command::CommandError::CommandFailed(TextComponent::text(
                    &format!("no handler registered for command id {command_id}"),
                )))
            },
            |handler| handler.handle(sender, server, args),
        )
    }

    /// WIT entry point — dispatches an incoming command suggestion request to the registered handler.
    fn handle_command_suggestion(
        handler_id: u32,
        sender: command::CommandSender,
        server: Server,
        request: command::SuggestionRequest,
    ) -> command::CommandSuggestions {
        let handlers = COMMAND_SUGGESTION_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(handler) = handlers.get(&handler_id) {
            handler.suggest(sender, server, request)
        } else {
            command::CommandSuggestions {
                start: request.start,
                length: 0,
                values: Vec::new(),
            }
        }
    }

    /// WIT entry point — dispatches a scheduled task invocation to the registered handler for `handler_id`.
    fn handle_task(handler_id: u32, server: Server) {
        let mut handlers = TASK_HANDLERS.lock().unwrap_or_else(|e| e.into_inner());
        handlers.handle(handler_id, server);
    }

    fn handle_ai_goal_can_start(goal_id: u32, server: Server, entity: entity::Entity) -> bool {
        let mut handlers = crate::ai::AI_GOAL_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(goal) = handlers.handlers.get_mut(&goal_id) {
            goal.can_start(server, entity)
        } else {
            false
        }
    }

    fn handle_ai_goal_should_continue(
        goal_id: u32,
        server: Server,
        entity: entity::Entity,
    ) -> bool {
        let mut handlers = crate::ai::AI_GOAL_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(goal) = handlers.handlers.get_mut(&goal_id) {
            goal.should_continue(server, entity)
        } else {
            false
        }
    }

    fn handle_ai_goal_start(goal_id: u32, server: Server, entity: entity::Entity) {
        let mut handlers = crate::ai::AI_GOAL_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(goal) = handlers.handlers.get_mut(&goal_id) {
            goal.start(server, entity);
        }
    }

    fn handle_ai_goal_tick(goal_id: u32, server: Server, entity: entity::Entity) {
        let mut handlers = crate::ai::AI_GOAL_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(goal) = handlers.handlers.get_mut(&goal_id) {
            goal.tick(server, entity);
        }
    }

    fn handle_ai_goal_stop(goal_id: u32, server: Server, entity: entity::Entity) {
        let mut handlers = crate::ai::AI_GOAL_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(goal) = handlers.handlers.get_mut(&goal_id) {
            goal.stop(server, entity);
        }
    }

    fn handle_ipc_message(
        sender: wit::PluginId,
        message: wit::IpcMessage,
    ) -> Result<wit::IpcMessage, String> {
        plugin().handle_ipc_message(sender, message)
    }

    fn handle_generate_phase(
        generator_id: u32,
        phase: wit::pumpkin::plugin::world::GenerationPhase,
        chunk: wit::pumpkin::plugin::world::ChunkBuffer,
    ) {
        let handlers = crate::worldgen::GENERATOR_HANDLERS
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(generator) = handlers.get(&generator_id) {
            let mut buffer = crate::worldgen::ChunkBuffer::new(chunk);
            match phase {
                wit::pumpkin::plugin::world::GenerationPhase::Biomes => {
                    generator.generate_biomes(&mut buffer);
                }
                wit::pumpkin::plugin::world::GenerationPhase::Noise => {
                    generator.generate_noise(&mut buffer);
                }
                wit::pumpkin::plugin::world::GenerationPhase::Surface => {
                    generator.generate_surface(&mut buffer);
                }
                wit::pumpkin::plugin::world::GenerationPhase::Features => {
                    generator.generate_features(&mut buffer);
                }
            }
        }
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

    /// Called when the plugin receives a message from another plugin.
    fn handle_ipc_message(
        &mut self,
        _sender: wit::PluginId,
        _message: wit::IpcMessage,
    ) -> Result<wit::IpcMessage, String> {
        Err("This plugin cannot receive messages.".to_string())
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
    #[allow(clippy::expect_used)]
    unsafe {
        PLUGIN
            .as_deref_mut()
            .expect("PLUGIN must be initialized with register_plugin before use")
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
/// AI and mob goal utilities.
pub mod ai;
/// Persistent custom data containers (Bukkit-style `PersistentDataHolder`).
pub mod persistent_data;
pub use persistent_data::PersistentDataHolder;
/// Game rules definitions and values.
pub use wit::pumpkin::plugin::game_rules::{GameRule, GameRuleValue};
