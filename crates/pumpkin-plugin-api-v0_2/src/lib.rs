//! Rust SDK for plugins targeting `pumpkin:plugin@0.2.0`.
//!
//! Implement [`Plugin`] and export it with [`register_plugin!`]. Register handlers in
//! [`Plugin::on_load`]; [`Plugin::new`] and [`Plugin::metadata`] stay synchronous.
//! The separate `pumpkin-plugin-api` crate targets v0.1.
//!
//! Generated imports are available through [`host`]. Their signatures follow the WIT:
//! await async calls. Constructors and event/task registration remain synchronous.
//! Recipe and enchantment registration helpers are async, as are the typed methods on
//! [`PersistentDataHolder`]. Initialization installs the SDK's `tracing` subscriber.
//!
//! A suspended callback may overlap another invocation in the same component. Do not
//! hold locks or mutable borrows across `.await`. Callback futures need not be `Send`.
//! Dropping a future does not undo host operations that have already completed.
//!
//! ## Early Development Warning
//! This SDK is in preview. Event-resource integration and the complete event catalogue
//! are still pending. Build components for `wasm32-wasip2` with release mode or at least
//! optimization level 1; the current event variant fails validation without optimization.
//!

use std::sync::{Arc, OnceLock};

pub use registry::LocalBoxFuture;

pub mod block;
pub mod datapack;
pub mod enchantment;
mod ext;
pub mod forms;
pub mod inventory;
pub mod item;
mod keys;
pub mod logging;
pub mod mobs;
pub mod permissions;
pub mod persistent_data;
pub mod recipe;
pub mod team;

pub mod ai;
pub mod commands;
pub mod display;
pub mod events;
pub mod gametest;
mod registry;
pub mod scheduler;
pub mod worldgen;

/// The generated `pumpkin:plugin@0.2.0` guest bindings
#[allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::too_many_arguments,
    missing_docs
)]
pub mod wit {
    wit_bindgen::generate!({
        path: "../pumpkin-plugin-wit/v0.2",
        world: "plugin",
        pub_export_macro: true,
        chainable_methods: [
            "pumpkin:plugin/command@0.2.0#command",
            "pumpkin:plugin/command@0.2.0#command-node",
            "pumpkin:plugin/text@0.2.0#text-component"
        ]
    });
}

/// Host interfaces, with the async and synchronous functions the WIT declares
pub use wit::pumpkin::plugin as host;

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
    context::{self, MarketplaceMetadata},
    damage_types as damage_types_wit, data_components, datapack as datapack_wit,
    display as display_wit, enchantments as enchantments_wit, entity,
    entity_statuses as entity_statuses_wit,
    entity_types::EntityType,
    event as events_wit, game_events as game_events_wit, gui, i18n, inventory as inventory_wit,
    ipc, item_stack, java_dialogs, java_packets, particles, permission, player,
    potions as potions_wit, recipe as recipe_wit, scoreboard, screens as screens_wit, server,
    statistics as statistics_wit, uuid, world,
};

// Convenience re-exports of commonly-used plugin types so plugin authors can
// name them directly (e.g. build an `ItemStack` for a GUI or `/give`).
pub use block::{BlockStateTypeExt, BlockType, BlockTypeExt, IntoBlockKey};
pub use damage_types_wit::DamageType;
pub use datapack::{DatapackInfo, DatapackManager, EnablePosition};
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
pub use entity_statuses_wit::EntityStatus;
pub use events::{EventHandler, FromIntoEvent};
pub use ext::player::{PlayerCooldownExt, PlayerEnderChestExt};
pub use game_events_wit::GameEvent;
pub use host::game_rules::{GameRule, GameRuleValue};
pub use host::world::{ChunkBuffer, GenerationPhase};
pub use inventory::{Inventory, PlayerInventory};
pub use item::{IntoItemKey, Item, ItemStackExt};
pub use mobs::{
    Ageable, AgeableData, Cat, CatData, Creeper, CreeperData, DyeColor, Enderman, EndermanData,
    EntityCastExt, Fox, FoxData, IronGolem, IronGolemData, MobCast, MobData, Sheep, SheepData,
    Shulker, ShulkerData, Slime, SlimeData, Villager, VillagerData, VillagerProfession, Wolf,
    WolfData, Zombie, ZombieData,
};
pub use persistent_data::PersistentDataHolder;
pub use potions_wit::PotionType;
pub use recipe::{
    CookingRecipeBuilder, Ingredient, RecipeCategory, RecipeError, RecipeManager,
    RegistrableRecipe, ShapedRecipeBuilder, ShapelessRecipeBuilder,
};
pub use screens_wit::Screen;
pub use statistics_wit::{CustomStatistic, StatisticCategory};
pub use team::{PlayerTeamExt, ScoreboardTeamExt, Team, TeamSettingsBuilder};
pub use wit::pumpkin::plugin::attributes::{Attribute, AttributeModifier, ModifierOperation};
pub use wit::pumpkin::plugin::item_stack::{ItemAttributeModifier, ItemStack};
pub use wit::pumpkin::plugin::player::{
    BedrockMinecraftVersion, BedrockPlayer, JavaMinecraftVersion, JavaPlayer, Player,
};
pub use wit::pumpkin::plugin::scoreboard::{CollisionRule, NametagVisibility, TeamSettings};
pub use wit::pumpkin::plugin::server::Dimension;
pub use wit::pumpkin::plugin::world::{
    Block, BlockDirection, BlockState, BlockStateInfo, Flammable, LivingEntity, Mob, PathNodeType,
    RayTraceBlockResult, RayTraceEntityResult, RaycastResult, World, WorldBorder,
};
pub use worldgen::{ChunkGenerator, GeneratorManager, register_chunk_generator};

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

// The types most plugins name directly
pub use host::{
    context::Context,
    event::{Event, EventPriority, EventType},
    ipc::{IpcMessage, PluginId},
    server::Server,
    text::{self, TextComponent},
    world::Entity,
};

/// The result type used by plugin callbacks
pub type Result<T, E = String> = core::result::Result<T, E>;

/// Describes a plugin to the server
pub struct PluginMetadata {
    /// The plugin name, which other plugins use as the recipient of an IPC message
    pub name: String,
    /// The version of the plugin itself; it does not select the ABI
    pub version: String,
    /// The plugin authors
    pub authors: Vec<String>,
    /// A short description of the plugin
    pub description: String,
    /// Names of the plugins this one depends on, which the host loads first
    pub dependencies: Vec<String>,
    /// The permissions the plugin requests
    pub permissions: Vec<String>,
}

/// The entry point of a v0.2 plugin
///
/// Implement it on one type and hand that type to [`register_plugin!`]; the host creates it once
/// and then drives it through the lifecycle in the [crate docs](crate#life-of-a-plugin)
///
/// Only [`new`](Self::new) and [`metadata`](Self::metadata) are required
#[allow(
    async_fn_in_trait,
    unused_variables,
    reason = "guest futures are intentionally not Send and defaults ignore their arguments"
)]
pub trait Plugin: Send + Sync + 'static {
    /// Creates the plugin
    ///
    /// The host calls this once from the synchronous `init-plugin` export for bootstrap wiring
    /// The [`Context`] and awaitable setup belong in [`on_load`](Self::on_load)
    fn new() -> Self
    where
        Self: Sized;

    /// Returns a pure snapshot of the plugin metadata
    ///
    /// The host reads this once, right after `init-plugin`; it is a synchronous export that
    /// returns by value, so it must not call host imports
    fn metadata(&self) -> PluginMetadata;

    /// Called when the plugin is loaded
    ///
    /// This is where handlers are registered, through the [`Context`] for events, commands and
    /// permissions and through the [`scheduler`], [`ai`] and [`worldgen`] modules for the rest
    ///
    /// # Errors
    ///
    /// An error fails the load: the host reports it as an initialization failure, calls
    /// [`on_unload`](Self::on_unload) and discards the plugin
    async fn on_load(&self, context: Context) -> Result<()> {
        Ok(())
    }

    /// Called before the plugin goes away
    ///
    /// The host also calls this after a failed [`on_load`](Self::on_load), so it has to cope with
    /// registrations that are only partly done
    ///
    /// # Errors
    ///
    /// The host ignores the result at every call site, so an error here is dropped
    async fn on_unload(&self, context: Context) -> Result<()> {
        Ok(())
    }

    /// Called when another plugin sends this one a message with [`host::ipc::send_ipc_message`]
    ///
    /// `sender` is the name of the sending plugin, and the sender awaits the reply; the default
    /// rejects every message
    ///
    /// # Errors
    ///
    /// The error string reaches the sender as the inner `Err` of its `send_ipc_message` call
    async fn handle_ipc_message(
        &self,
        sender: PluginId,
        message: IpcMessage,
    ) -> Result<IpcMessage, String> {
        Err("this plugin cannot receive messages".to_string())
    }
}

trait ErasedPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn on_load(&self, context: Context) -> LocalBoxFuture<'_, Result<()>>;
    fn on_unload(&self, context: Context) -> LocalBoxFuture<'_, Result<()>>;
    fn handle_ipc_message(
        &self,
        sender: PluginId,
        message: IpcMessage,
    ) -> LocalBoxFuture<'_, Result<IpcMessage, String>>;
}

impl<P: Plugin> ErasedPlugin for P {
    fn metadata(&self) -> PluginMetadata {
        Plugin::metadata(self)
    }

    fn on_load(&self, context: Context) -> LocalBoxFuture<'_, Result<()>> {
        Box::pin(Plugin::on_load(self, context))
    }

    fn on_unload(&self, context: Context) -> LocalBoxFuture<'_, Result<()>> {
        Box::pin(Plugin::on_unload(self, context))
    }

    fn handle_ipc_message(
        &self,
        sender: PluginId,
        message: IpcMessage,
    ) -> LocalBoxFuture<'_, Result<IpcMessage, String>> {
        Box::pin(Plugin::handle_ipc_message(self, sender, message))
    }
}

static PLUGIN: OnceLock<Arc<dyn ErasedPlugin>> = OnceLock::new();

// The plugin is cloned out of the cell, so no borrow of the cell outlives the lookup
fn plugin() -> Arc<dyn ErasedPlugin> {
    #[allow(
        clippy::expect_used,
        reason = "the host calls init-plugin before any callback"
    )]
    PLUGIN
        .get()
        .cloned()
        .expect("init-plugin must run before other plugin callbacks")
}

/// The functions [`register_plugin!`] forwards every export to, not part of the public API
///
/// Each one is a thin hop into a table in this crate; keeping them here instead of in the macro
/// means the logic is compiled once and the expansion stays easy to read
#[doc(hidden)]
pub mod runtime {
    use super::{
        Arc, Context, IpcMessage, PLUGIN, Plugin, PluginId, PluginMetadata, Result, plugin,
    };
    use crate::{
        Event, Server, ai, commands, events,
        host::{
            command::{
                CommandError, CommandSender, CommandSuggestions, ConsumedArgs, SuggestionRequest,
            },
            world::{ChunkBuffer, Entity, GenerationPhase},
        },
        scheduler, worldgen,
    };

    pub use crate::gametest::{
        invoke_async_test, invoke_block_predicate, invoke_entity_predicate, invoke_test,
        invoke_void,
    };

    pub fn init_plugin<P: Plugin>() {
        let _ = tracing::subscriber::set_global_default(crate::logging::WitSubscriber::new());
        assert!(
            PLUGIN.set(Arc::new(P::new())).is_ok(),
            "init-plugin must only run once"
        );
    }

    #[must_use]
    pub fn metadata() -> PluginMetadata {
        plugin().metadata()
    }

    pub async fn on_load(context: Context) -> Result<()> {
        plugin().on_load(context).await
    }

    pub async fn on_unload(context: Context) -> Result<()> {
        plugin().on_unload(context).await
    }

    pub async fn handle_event(event_id: u32, server: Server, event: Event) -> Event {
        events::dispatch(event_id, server, event).await
    }

    pub async fn handle_command(
        command_id: u32,
        sender: CommandSender,
        server: Server,
        args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        commands::dispatch_command(command_id, sender, server, args).await
    }

    pub async fn handle_command_suggestion(
        handler_id: u32,
        sender: CommandSender,
        server: Server,
        request: SuggestionRequest,
    ) -> CommandSuggestions {
        commands::dispatch_suggestion(handler_id, sender, server, request).await
    }

    pub async fn handle_task(handler_id: u32, server: Server) {
        scheduler::dispatch(handler_id, server).await;
    }

    pub async fn handle_ipc_message(
        sender: PluginId,
        message: IpcMessage,
    ) -> Result<IpcMessage, String> {
        plugin().handle_ipc_message(sender, message).await
    }

    pub async fn handle_ai_goal_can_start(goal_id: u32, server: Server, entity: Entity) -> bool {
        ai::can_start(goal_id, server, entity).await
    }

    pub async fn handle_ai_goal_should_continue(
        goal_id: u32,
        server: Server,
        entity: Entity,
    ) -> bool {
        ai::should_continue(goal_id, server, entity).await
    }

    pub async fn handle_ai_goal_start(goal_id: u32, server: Server, entity: Entity) {
        ai::start(goal_id, server, entity).await;
    }

    pub async fn handle_ai_goal_tick(goal_id: u32, server: Server, entity: Entity) {
        ai::tick(goal_id, server, entity).await;
    }

    pub async fn handle_ai_goal_stop(goal_id: u32, server: Server, entity: Entity) {
        ai::stop(goal_id, server, entity).await;
    }

    pub async fn handle_generate_phase(
        generator_id: u32,
        phase: GenerationPhase,
        chunk: ChunkBuffer,
    ) {
        worldgen::dispatch(generator_id, phase, chunk).await;
    }
}

/// Registers a type as the plugin and exports the v0.2 world from the final component
///
/// Call it once, at the top level of the plugin crate, with a type that implements [`Plugin`]
///
/// # Why this is a macro
///
/// `init-plugin` has to construct your plugin type, and the generated `Guest` traits are
/// implemented in the crate that exports them
///
/// The macro implements the three `Guest` traits of the world (the callbacks, `metadata` and the
/// `GameTest` callbacks) on a private type and exports it, so no symbol has to be resolved at
/// link time; each method forwards to this crate's tables and holds no logic of its own
///
/// ```rust,ignore
/// register_plugin!(MyPlugin);
/// ```
#[macro_export]
macro_rules! register_plugin {
    ($plugin_type:ty) => {
        const _: () = {
            struct PumpkinPluginComponent;

            impl $crate::wit::Guest for PumpkinPluginComponent {
                fn init_plugin() {
                    $crate::runtime::init_plugin::<$plugin_type>();
                }

                async fn on_load(
                    context: $crate::host::context::Context,
                ) -> ::core::result::Result<(), ::std::string::String> {
                    $crate::runtime::on_load(context).await
                }

                async fn on_unload(
                    context: $crate::host::context::Context,
                ) -> ::core::result::Result<(), ::std::string::String> {
                    $crate::runtime::on_unload(context).await
                }

                async fn handle_event(
                    event_id: u32,
                    server: $crate::host::server::Server,
                    event: $crate::host::event::Event,
                ) -> $crate::host::event::Event {
                    $crate::runtime::handle_event(event_id, server, event).await
                }

                async fn handle_command(
                    command_id: u32,
                    sender: $crate::host::command::CommandSender,
                    server: $crate::host::server::Server,
                    args: $crate::host::command::ConsumedArgs,
                ) -> ::core::result::Result<i32, $crate::host::command::CommandError> {
                    $crate::runtime::handle_command(command_id, sender, server, args).await
                }

                async fn handle_command_suggestion(
                    handler_id: u32,
                    sender: $crate::host::command::CommandSender,
                    server: $crate::host::server::Server,
                    request: $crate::host::command::SuggestionRequest,
                ) -> $crate::host::command::CommandSuggestions {
                    $crate::runtime::handle_command_suggestion(handler_id, sender, server, request)
                        .await
                }

                async fn handle_task(handler_id: u32, server: $crate::host::server::Server) {
                    $crate::runtime::handle_task(handler_id, server).await;
                }

                async fn handle_ipc_message(
                    sender: $crate::host::ipc::PluginId,
                    message: $crate::host::ipc::IpcMessage,
                ) -> ::core::result::Result<$crate::host::ipc::IpcMessage, ::std::string::String>
                {
                    $crate::runtime::handle_ipc_message(sender, message).await
                }

                async fn handle_ai_goal_can_start(
                    goal_id: u32,
                    server: $crate::host::server::Server,
                    entity: $crate::host::world::Entity,
                ) -> bool {
                    $crate::runtime::handle_ai_goal_can_start(goal_id, server, entity).await
                }

                async fn handle_ai_goal_should_continue(
                    goal_id: u32,
                    server: $crate::host::server::Server,
                    entity: $crate::host::world::Entity,
                ) -> bool {
                    $crate::runtime::handle_ai_goal_should_continue(goal_id, server, entity).await
                }

                async fn handle_ai_goal_start(
                    goal_id: u32,
                    server: $crate::host::server::Server,
                    entity: $crate::host::world::Entity,
                ) {
                    $crate::runtime::handle_ai_goal_start(goal_id, server, entity).await;
                }

                async fn handle_ai_goal_tick(
                    goal_id: u32,
                    server: $crate::host::server::Server,
                    entity: $crate::host::world::Entity,
                ) {
                    $crate::runtime::handle_ai_goal_tick(goal_id, server, entity).await;
                }

                async fn handle_ai_goal_stop(
                    goal_id: u32,
                    server: $crate::host::server::Server,
                    entity: $crate::host::world::Entity,
                ) {
                    $crate::runtime::handle_ai_goal_stop(goal_id, server, entity).await;
                }

                async fn handle_generate_phase(
                    generator_id: u32,
                    phase: $crate::host::world::GenerationPhase,
                    chunk: $crate::host::world::ChunkBuffer,
                ) {
                    $crate::runtime::handle_generate_phase(generator_id, phase, chunk).await;
                }
            }

            impl $crate::wit::exports::pumpkin::plugin::metadata::Guest for PumpkinPluginComponent {
                fn get_metadata() -> $crate::wit::exports::pumpkin::plugin::metadata::PluginMetadata
                {
                    let metadata = $crate::runtime::metadata();
                    $crate::wit::exports::pumpkin::plugin::metadata::PluginMetadata {
                        name: metadata.name,
                        version: metadata.version,
                        authors: metadata.authors,
                        description: metadata.description,
                        dependencies: metadata.dependencies,
                        permissions: metadata.permissions,
                    }
                }
            }

            impl $crate::wit::exports::pumpkin::plugin::gametest_callbacks::Guest
                for PumpkinPluginComponent
            {
                fn invoke_void(callback: $crate::gametest::VoidCallbackId) {
                    $crate::runtime::invoke_void(&callback);
                }

                fn invoke_test(
                    callback: $crate::gametest::TestCallbackId,
                    test: $crate::gametest::Test,
                ) {
                    $crate::runtime::invoke_test(&callback, test);
                }

                async fn invoke_async_test(
                    callback: $crate::gametest::AsyncTestCallbackId,
                    test: $crate::gametest::Test,
                ) {
                    $crate::runtime::invoke_async_test(&callback, test).await;
                }

                fn invoke_block_predicate(
                    callback: $crate::gametest::BlockPredicateCallbackId,
                    permutation: $crate::gametest::BlockPermutation,
                ) -> bool {
                    $crate::runtime::invoke_block_predicate(&callback, permutation)
                }

                fn invoke_entity_predicate(
                    callback: $crate::gametest::EntityPredicateCallbackId,
                    entity: &$crate::host::world::Entity,
                ) -> bool {
                    $crate::runtime::invoke_entity_predicate(&callback, entity)
                }
            }

    $crate::wit::export!(PumpkinPluginComponent with_types_in $crate::wit);
        };
    };
}
