//! Guest SDK for the async `pumpkin:plugin@0.2.0` world, in preview
//!
//! Depend on this crate when you want plugin handlers that can `.await` host calls
//!
//! It sits next to `pumpkin-plugin-api` instead of replacing it; that crate still targets the
//! synchronous `pumpkin:plugin@0.1.0` world and keeps working for every v0.1 plugin
//!
//! # Where this fits
//!
//! A plugin is a WebAssembly component and the server is the host around it; the WIT files in
//! `crates/pumpkin-plugin-wit/v0.2` are the contract between the two, and this crate is the
//! guest half of that contract
//!
//! ```text
//!  host                                      guest (your plugin)
//!  ----                                      -------------------
//!  1. handle-event(7, server, event)  -->    events::dispatch finds handler 7
//!  2.                                        your handler runs up to its first .await
//!  3. <-- player.has-permission(node)        an async import, so the guest suspends here
//!  4. does the work and replies       -->    the handler resumes with the answer
//!  5. <-- returns the owned event            applied for a blocking handler, ignored otherwise
//! ```
//!
//! The 14 exports are the callbacks the host can start: `init-plugin`, `on-load`, `on-unload`,
//! events, commands, command suggestions, tasks, IPC messages, five AI goal hooks and chunk
//! generation; this crate wires each one to a table you fill from `on_load`
//!
//! How the server schedules those calls, which Store they run in and what limits apply is the
//! host's business; nothing in this crate knows about it, and nothing here should
//!
//! # Why a second crate
//!
//! A component contains exactly one world, so a plugin is either v0.1 or v0.2 and never both
//!
//! A cargo feature looked tempting and was rejected: features are additive across a workspace, so
//! any dependency that turned on `v0_2` would flip the ABI for everything else in the build
//!
//! The host reads the ABI from the WIT package the component imports and exports
//! (`pumpkin:plugin@0.1.0` or `pumpkin:plugin@0.2.0`) and never from [`PluginMetadata::version`],
//! which is only the version of your plugin
//!
//! # What is async and what is not
//!
//! The WIT marks a function `async func` only when the host implementation awaits something (an
//! async lock, I/O or another plugin), so those are exactly the calls you `.await`
//!
//! Everything else stays a plain function on purpose: resource constructors, `register_event`,
//! task registration, the `metadata` snapshot and most `GameTest` callbacks never wait, and
//! returning futures from them would only add ceremony; the generated bindings follow the WIT
//! one to one, so the compiler tells you which is which
//!
//! A synchronous export can never suspend whatever you write inside it, because an `async fn`
//! cannot make a synchronous component callback wait; that is why the synchronous `GameTest`
//! callbacks take plain closures
//!
//! # Life of a plugin
//!
//! 1. the host instantiates the component and calls `init-plugin`, which runs [`Plugin::new`]
//! 2. it reads [`Plugin::metadata`], a pure snapshot that must not call the host
//! 3. it calls [`Plugin::on_load`] with a [`Context`], and this is where you register handlers
//! 4. as the server runs it calls your handlers by id
//! 5. it calls [`Plugin::on_unload`] before the plugin goes away
//!
//! ```rust,ignore
//! use pumpkin_plugin_api_v0_2::{
//!     Context, EventPriority, Plugin, PluginMetadata, Result, Server,
//!     events::{EventData, EventHandler, PlayerChatEvent},
//!     register_plugin,
//! };
//!
//! struct ChatGuard;
//!
//! impl EventHandler<PlayerChatEvent> for ChatGuard {
//!     async fn handle(
//!         &self,
//!         _server: Server,
//!         mut event: EventData<PlayerChatEvent>,
//!     ) -> EventData<PlayerChatEvent> {
//!         // has_permission is an async import, so this suspends here and resumes with the answer
//!         if !event.player.has_permission("chat.send".to_string()).await {
//!             event.cancelled = true;
//!         }
//!         event
//!     }
//! }
//!
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn new() -> Self {
//!         Self
//!     }
//!
//!     fn metadata(&self) -> PluginMetadata {
//!         PluginMetadata {
//!             name: "chat-guard".into(),
//!             version: "0.1.0".into(),
//!             authors: vec![],
//!             description: "Cancels chat from players without chat.send".into(),
//!             dependencies: vec![],
//!             permissions: vec![],
//!         }
//!     }
//!
//!     async fn on_load(&self, context: Context) -> Result<()> {
//!         context.register_event_handler::<PlayerChatEvent, _>(
//!             ChatGuard,
//!             EventPriority::Normal,
//!             true,
//!         );
//!         Ok(())
//!     }
//! }
//!
//! register_plugin!(MyPlugin);
//! ```
//!
//! # How the host finds your handler
//!
//! Handlers live in the guest, so the guest picks their ids: registering one stores it in a table
//! here and tells the host only the number, for example `register-event(7, ...)`
//!
//! Later the host calls back with that number and the table finds the handler; there is one table
//! per kind (events, commands, suggestions, tasks, AI goals, generators and each `GameTest`
//! callback), so an id only means something inside its own table
//!
//! # Suspension and reentry
//!
//! A callback only yields at an explicit `.await` on a host import or on a future built from one;
//! CPU-bound code between awaits never yields, so split long work at awaits or keep it small
//!
//! While a callback is suspended the host may start another one in the same component instance,
//! even for the same handler; that is why handlers take `&self` and shared mutable state needs
//! interior mutability
//!
//! The one rule to remember is to never keep a `std::sync` guard or a `RefCell` borrow alive
//! across an await: the next callback can try to take it, and then neither can make progress
//!
//! The tables follow that rule themselves; they lock only inside short synchronous methods and
//! hand the dispatcher an owned `Arc` of the handler, so no guard exists while a handler runs
//!
//! # Ownership
//!
//! Callback arguments and results are owned values and resource handles: the event, command
//! arguments, entity, chunk buffer and server handle all move into the callback and stay valid
//! across every suspension
//!
//! The only borrows left in the ABI are the synchronous `GameTest` entity predicate and imports that
//! declare `borrow`, and neither can cross an await
//!
//! Passing an owned handle to a host import transfers it, and a conversion that can fail hands it
//! back, as [`display::EntityDisplayExt`] does with the original entity in the error
//!
//! # Cancellation, dropped futures and traps
//!
//! The guest cannot cancel host work, and a dropped waiter does not cancel guest work
//!
//! - if the host cancels an exported callback while it is suspended, the guest future is dropped
//!   at that await and its destructors run; nothing after that await executes, and work already
//!   handed to the host is not undone
//! - if a guest future awaiting a host import is dropped, the bindings ask the host to cancel that
//!   import; the request can lose the race with completion, so the import may already have taken
//!   effect and its result is thrown away
//! - dropping a future that was created but never polled does nothing; to run work to completion
//!   whatever its caller does, await it inside the callback
//! - a trap raised by a host import ends the component instance, so guest code never observes it,
//!   destructors do not run and later calls into the plugin fail; treat it as fatal for the plugin
//! - a timeout enforced by the host or by a caller waiting on a plugin only stops the wait, and the
//!   guest callback keeps running until it finishes or the host cancels it as described above
//!
//! # Building and checking a component
//!
//! ```text
//! cargo build --release --target wasm32-wasip2
//! wasm-tools component wit target/wasm32-wasip2/release/my_plugin.wasm
//! ```
//!
//! The second command prints the world the component really declares, so look for
//! `package pumpkin:plugin@0.2.0` and `async func` on the exports
//!
//! Build with `--release` or at least opt-level 1; an unoptimized build of the lifted `event`
//! variant goes past the Wasm locals limit and fails validation
//!
//! # What is not here yet
//!
//! This is the foundation, so it is deliberately small: only a few event markers are defined
//! (see [`events`]) and [`define_event!`](crate::define_event) makes the rest a one-liner, while the helper surface of
//! `pumpkin-plugin-api` (permission constants, the logging subscriber, item and block builders)
//! has not been ported and ergonomics and a migration guide come later

use std::sync::{Arc, OnceLock};

pub use registry::LocalBoxFuture;

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

// The types most plugins name directly
pub use host::{
    context::Context,
    event::{Event, EventPriority, EventType},
    ipc::{IpcMessage, PluginId},
    server::Server,
    text,
    world::Entity,
};

/// The result type used by plugin callbacks
pub type Result<T, E = String> = core::result::Result<T, E>;

/// Metadata that describes a plugin to the server
pub struct PluginMetadata {
    /// The human-readable name of the plugin
    pub name: String,
    /// The version string of the plugin
    pub version: String,
    /// The plugin authors
    pub authors: Vec<String>,
    /// A short description of the plugin
    pub description: String,
    /// The plugin dependencies
    pub dependencies: Vec<String>,
    /// The permissions requested by the plugin
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
    /// The host calls this once, from the async `init-plugin` export, before anything else in the
    /// plugin runs; it is synchronous, and the [`Context`] only arrives later in
    /// [`on_load`](Self::on_load)
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
/// `init-plugin` is an async export now, and an async-lifted export cannot be the hand-written
/// `extern "C"` function that v0.1 used; the bindings have to generate it, and generated exports
/// belong to the crate that ends up in the final component, which is yours
///
/// So the macro implements the three `Guest` traits of the world (the callbacks, `metadata` and
/// the `GameTest` callbacks) on a private type and exports it; each method is a one-line forward
/// into this crate's tables, so there is no logic hiding in the expansion
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
                async fn init_plugin() {
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
