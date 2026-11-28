#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::significant_drop_in_scrutinee)]
// Not warn event sending macros
#![allow(unused_labels, deprecated)]

use std::path::Path;
use std::sync::Arc;

use pumpkin_wasm_host_common::state::FromResource;

use crate::bindings::PluginPre;
use pumpkin_core::plugin::PluginMetadata;
use pumpkin_wasm_host_common::{
    concurrent_store::{LegacyGuestScope, LegacySyncReentry},
    plugin::PluginInitError,
    state::PluginHostState,
};
use wasmtime::component::{Accessor, HasSelf, InstancePre, Linker, Resource};
use wasmtime::{Engine, Store};

mod bindings;

pub mod advancement;
pub mod block_entity;
pub mod boss_bar;
pub mod commands;
pub mod common;
pub mod context;
pub mod datapack;
pub mod display;
pub mod enchantment;
pub mod entity;
pub mod events;
pub mod forms;
pub mod gametest;
pub mod generated_packets;
pub mod gui;
pub mod i18n;
pub mod inventory;
pub mod ipc;
pub mod item_stack;
pub mod java_dialogs;
pub mod living_entity;
pub mod logging;
pub mod mob;
pub mod permission;
pub mod player;
pub mod recipe;
pub mod scheduler;
pub mod scoreboard;
pub mod server;
pub mod status_effect;
pub mod text;
pub mod uuid;
pub mod world;

pub use crate::bindings::pumpkin;

/// A v0.2 plugin instance backed either by a Wasmtime component or a native (`cdylib`) WIT library.
pub enum Plugin {
    Wasm(Box<bindings::Plugin>),
    Native(Arc<bindings::native::Plugin>),
}

impl Plugin {
    #[must_use]
    pub const fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    #[must_use]
    pub fn native_library(&self) -> Option<Arc<pumpkin_native_wit::libloading::Library>> {
        match self {
            Self::Wasm(_) => None,
            Self::Native(plugin) => Some(plugin.library()),
        }
    }

    pub fn attach_concurrent_store(
        &self,
        runner: Arc<dyn pumpkin_native_wit::ConcurrentStoreRunner<PluginHostState>>,
    ) {
        if let Self::Native(plugin) = self {
            plugin.attach_concurrent_store(runner);
        }
    }

    pub async fn call_init_plugin(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => guest.call(plugin.func_init_plugin(), ()).await,
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || plugin.call_init_plugin())
                    .await
            }
        }
    }

    pub async fn call_get_metadata(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
    ) -> wasmtime::Result<PluginMetadata> {
        let metadata = match self {
            Self::Wasm(plugin) => {
                guest
                    .call(plugin.pumpkin_plugin_metadata().func_get_metadata(), ())
                    .await?
                    .0
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || plugin.call_get_metadata())
                    .await?
            }
        };
        Ok(PluginMetadata {
            name: metadata.name,
            version: metadata.version,
            authors: metadata.authors,
            description: metadata.description,
            dependencies: metadata.dependencies,
            permissions: metadata.permissions,
        })
    }

    pub async fn call_on_load(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        context: Resource<pumpkin::plugin::context::Context>,
    ) -> wasmtime::Result<Result<(), String>> {
        match self {
            Self::Wasm(plugin) => guest
                .call(plugin.func_on_load(), (context,))
                .await
                .map(|(result,)| result),
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || plugin.call_on_load(context))
                    .await
            }
        }
    }

    pub async fn call_on_unload(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        context: Resource<pumpkin::plugin::context::Context>,
    ) -> wasmtime::Result<Result<(), String>> {
        match self {
            Self::Wasm(plugin) => guest
                .call(plugin.func_on_unload(), (context,))
                .await
                .map(|(result,)| result),
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || plugin.call_on_unload(context))
                    .await
            }
        }
    }

    pub async fn call_handle_ipc_message(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        sender: String,
        message: Vec<u8>,
    ) -> wasmtime::Result<Result<Vec<u8>, String>> {
        match self {
            Self::Wasm(plugin) => {
                let function = *plugin.func_handle_ipc_message().func();
                guest
                    .call_func::<(String, Vec<u8>), (Result<Vec<u8>, String>,)>(
                        function,
                        (sender, message),
                    )
                    .await
                    .map(|(result,)| result)
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_ipc_message(&sender, &message)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_task(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        handler_id: u32,
        server: Resource<pumpkin::plugin::server::Server>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                guest
                    .call(plugin.func_handle_task(), (handler_id, server))
                    .await
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || plugin.call_handle_task(handler_id, server))
                    .await
            }
        }
    }

    pub async fn call_handle_event(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        handler_id: u32,
        server: Resource<pumpkin::plugin::server::Server>,
        event: Resource<pumpkin::plugin::event::Event>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                let function = *plugin.func_handle_event().func();
                guest
                    .call_func::<(
                        u32,
                        Resource<pumpkin::plugin::server::Server>,
                        Resource<pumpkin::plugin::event::Event>,
                    ), ()>(function, (handler_id, server, event))
                    .await
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_event(handler_id, server, event)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_command(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        handler_id: u32,
        sender: Resource<pumpkin::plugin::command::CommandSender>,
        server: Resource<pumpkin::plugin::server::Server>,
        args: Resource<pumpkin::plugin::command::ConsumedArgs>,
    ) -> wasmtime::Result<Result<i32, pumpkin::plugin::command::CommandError>> {
        match self {
            Self::Wasm(plugin) => guest
                .call(
                    plugin.func_handle_command(),
                    (handler_id, sender, server, args),
                )
                .await
                .map(|(result,)| result),
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_command(handler_id, sender, server, args)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_command_suggestion(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        handler_id: u32,
        sender: Resource<pumpkin::plugin::command::CommandSender>,
        server: Resource<pumpkin::plugin::server::Server>,
        request: pumpkin::plugin::command::SuggestionRequest,
    ) -> wasmtime::Result<pumpkin::plugin::command::CommandSuggestions> {
        match self {
            Self::Wasm(plugin) => guest
                .call(
                    plugin.func_handle_command_suggestion(),
                    (handler_id, sender, server, request),
                )
                .await
                .map(|(result,)| result),
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_command_suggestion(handler_id, sender, server, &request)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_ai_goal_start(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        goal_id: u32,
        server: Resource<pumpkin::plugin::server::Server>,
        entity: Resource<pumpkin::plugin::world::Entity>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                guest
                    .call(
                        plugin.func_handle_ai_goal_start(),
                        (goal_id, server, entity),
                    )
                    .await
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_ai_goal_start(goal_id, server, entity)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_ai_goal_tick(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        goal_id: u32,
        server: Resource<pumpkin::plugin::server::Server>,
        entity: Resource<pumpkin::plugin::world::Entity>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                guest
                    .call(plugin.func_handle_ai_goal_tick(), (goal_id, server, entity))
                    .await
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_ai_goal_tick(goal_id, server, entity)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_ai_goal_stop(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        goal_id: u32,
        server: Resource<pumpkin::plugin::server::Server>,
        entity: Resource<pumpkin::plugin::world::Entity>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                guest
                    .call(plugin.func_handle_ai_goal_stop(), (goal_id, server, entity))
                    .await
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_ai_goal_stop(goal_id, server, entity)
                    })
                    .await
            }
        }
    }

    pub async fn call_handle_generate_phase(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        generator_id: u32,
        phase: pumpkin::plugin::world::GenerationPhase,
        buffer: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                guest
                    .call(
                        plugin.func_handle_generate_phase(),
                        (generator_id, phase, buffer),
                    )
                    .await
            }
            Self::Native(plugin) => {
                let plugin = Arc::clone(plugin);
                guest
                    .call_native_concurrent(move || {
                        plugin.call_handle_generate_phase(generator_id, phase, buffer)
                    })
                    .await
            }
        }
    }
}

fn active_plugin(
    accessor: &Accessor<PluginHostState, HasSelf<PluginHostState>>,
) -> wasmtime::Result<std::sync::Arc<pumpkin_wasm_host_common::plugin::WasmPlugin>> {
    accessor.with(|mut host| {
        host.get()
            .plugin
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
            .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))
    })
}

/// Runs synchronous server work on Tokio's blocking pool without holding a store borrow
///
/// Callbacks from `operation` inherit the active guest call's chain and root admission
/// The reentry depth is checked before the operation is spawned
pub(crate) async fn run_blocking<R: Send + 'static>(
    accessor: &Accessor<PluginHostState, HasSelf<PluginHostState>>,
    operation: impl FnOnce() -> R + Send + 'static,
) -> wasmtime::Result<R> {
    active_plugin(accessor)?
        .store
        .spawn_blocking_in_active_context(operation)
        .await
}

/// Awaits host work with the active guest call's chain and root admission
///
/// Use this when the operation can call back into plugins so callbacks stay in the same chain
/// The reentry depth is checked before the future is polled
pub(crate) async fn in_active_context<R>(
    accessor: &Accessor<PluginHostState, HasSelf<PluginHostState>>,
    future: impl std::future::Future<Output = R>,
) -> wasmtime::Result<R> {
    active_plugin(accessor)?
        .store
        .in_active_context(future)
        .await
}

/// Resource-table access from an [`Accessor`] without holding the store borrow across an
/// `.await`.
pub(crate) trait AccessorExt {
    fn get_res<T: FromResource>(&self, res: &Resource<T>) -> wasmtime::Result<T::Internal>
    where
        T::Internal: Clone;
    fn take_res<T: FromResource>(&self, res: Resource<T>) -> wasmtime::Result<T::Internal>;
    fn add_res<T: FromResource>(&self, item: T::Internal) -> wasmtime::Result<Resource<T>>;
    fn server(&self) -> wasmtime::Result<std::sync::Arc<pumpkin_core::server::Server>>;
}

impl AccessorExt for Accessor<PluginHostState, HasSelf<PluginHostState>> {
    fn get_res<T: FromResource>(&self, res: &Resource<T>) -> wasmtime::Result<T::Internal>
    where
        T::Internal: Clone,
    {
        self.with(|mut host| host.get().get(res).cloned())
    }

    fn take_res<T: FromResource>(&self, res: Resource<T>) -> wasmtime::Result<T::Internal> {
        self.with(|mut host| host.get().take(res))
    }

    fn add_res<T: FromResource>(&self, item: T::Internal) -> wasmtime::Result<Resource<T>> {
        self.with(|mut host| host.get().add(item))
    }

    fn server(&self) -> wasmtime::Result<std::sync::Arc<pumpkin_core::server::Server>> {
        self.with(|mut host| host.get().require_server().cloned())
    }
}

/// Server access for synchronous host imports, which work on the host state directly instead of
/// through an [`Accessor`].
pub(crate) trait RequireServer {
    /// Borrows the server, failing if the host state has none set.
    fn require_server(&self) -> wasmtime::Result<&std::sync::Arc<pumpkin_core::server::Server>>;
}

impl RequireServer for PluginHostState {
    fn require_server(&self) -> wasmtime::Result<&std::sync::Arc<pumpkin_core::server::Server>> {
        self.server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))
    }
}

/// Locking for a standard [`Mutex`](std::sync::Mutex) that ignores poisoning.
///
/// Returns the guard even after a panic has poisoned the mutex
/// The protected data is left as the panicking thread left it, with no validation or repair
pub(crate) trait LockIgnorePoison<T: ?Sized> {
    /// Locks the mutex, taking the guard out of the poison error if it was poisoned.
    fn lock_ignore_poison(&self) -> std::sync::MutexGuard<'_, T>;
}

impl<T: ?Sized> LockIgnorePoison<T> for std::sync::Mutex<T> {
    fn lock_ignore_poison(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Locking for a standard [`RwLock`](std::sync::RwLock) that ignores poisoning, see
/// [`LockIgnorePoison`].
pub(crate) trait RwLockIgnorePoison<T: ?Sized> {
    /// Takes a read lock, taking the guard out of the poison error if it was poisoned.
    fn read_ignore_poison(&self) -> std::sync::RwLockReadGuard<'_, T>;

    /// Takes a write lock, taking the guard out of the poison error if it was poisoned.
    fn write_ignore_poison(&self) -> std::sync::RwLockWriteGuard<'_, T>;
}

impl<T: ?Sized> RwLockIgnorePoison<T> for std::sync::RwLock<T> {
    fn read_ignore_poison(&self) -> std::sync::RwLockReadGuard<'_, T> {
        self.read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn write_ignore_poison(&self) -> std::sync::RwLockWriteGuard<'_, T> {
        self.write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

mod resource_with {
    use super::pumpkin::plugin;
    use pumpkin_core::{
        command::CommandSender,
        entity::{EntityBase, player::Player},
        plugin::{Context, QueuedEvent, api::gui::PluginGui},
        server::{RecipeManager, Server},
        world::World,
    };
    use pumpkin_util::text::TextComponent;
    use pumpkin_wasm_host_common::{
        args::OwnedArg,
        state::{
            ChunkBuffer, FromResource, InventoryProvider, ScoreboardProvider, WasmCommand,
            WasmCommandNode,
        },
    };
    use std::collections::HashMap;
    use std::sync::{Arc, Weak};
    use tokio::sync::Mutex;

    impl FromResource for plugin::server::Server {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::context::Context {
        type Internal = Arc<Context>;
    }
    impl FromResource for plugin::player::Player {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::player::JavaPlayer {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::player::BedrockPlayer {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::world::Entity {
        type Internal = Arc<dyn EntityBase>;
    }
    impl FromResource for plugin::world::World {
        type Internal = Arc<World>;
    }
    impl FromResource for plugin::world::Chunk {
        type Internal = (Arc<World>, Weak<pumpkin_world::chunk::ChunkData>);
    }
    impl FromResource for plugin::world::WorldBorder {
        type Internal = Arc<World>;
    }

    impl FromResource for plugin::scoreboard::Scoreboard {
        type Internal = ScoreboardProvider;
    }
    impl FromResource for plugin::scoreboard::BedrockScoreboard {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::gui::Gui {
        type Internal = Arc<Mutex<PluginGui>>;
    }
    impl FromResource for plugin::boss_bar::BossBar {
        type Internal = Arc<Mutex<crate::boss_bar::PluginBossBar>>;
    }
    impl FromResource for plugin::text::TextComponent {
        type Internal = TextComponent;
    }
    impl FromResource for plugin::command::Command {
        type Internal = WasmCommand;
    }
    impl FromResource for plugin::command::CommandSender {
        type Internal = CommandSender;
    }
    impl FromResource for plugin::command::ConsumedArgs {
        type Internal = HashMap<String, OwnedArg>;
    }
    impl FromResource for plugin::command::CommandNode {
        type Internal = WasmCommandNode;
    }
    impl FromResource for plugin::item_stack::ItemStack {
        type Internal = Arc<Mutex<pumpkin_data::item_stack::ItemStack>>;
    }
    impl FromResource for plugin::recipe::RecipeManager {
        type Internal = Arc<RecipeManager>;
    }
    impl FromResource for plugin::enchantments::EnchantmentManager {
        type Internal = Arc<pumpkin_core::server::enchantment::EnchantmentManager>;
    }
    impl FromResource for plugin::server::OpManager {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::server::BanManager {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::server::WhitelistManager {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::server::DatapackManager {
        type Internal = Arc<Server>;
    }

    #[allow(clippy::wildcard_imports)]
    mod block_entities {
        use super::super::pumpkin::plugin;
        use pumpkin_core::block::entities::*;
        use pumpkin_wasm_host_common::state::{ContainerBlockEntity, FromResource};
        use std::sync::Arc;

        impl FromResource for plugin::block_entity::BlockEntity {
            type Internal = Arc<dyn BlockEntity>;
        }
        impl FromResource for plugin::block_entity::BannerBlockEntity {
            type Internal = Arc<banner::BannerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BarrelBlockEntity {
            type Internal = Arc<barrel::BarrelBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BeaconBlockEntity {
            type Internal = Arc<beacon::BeaconBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BedBlockEntity {
            type Internal = Arc<bed::BedBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BeehiveBlockEntity {
            type Internal = Arc<beehive::BeehiveBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BellBlockEntity {
            type Internal = Arc<bell::BellBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BlastingFurnaceBlockEntity {
            type Internal = Arc<blasting_furnace::BlastingFurnaceBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BrewingStandBlockEntity {
            type Internal = Arc<brewing_stand::BrewingStandBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BrushableBlockBlockEntity {
            type Internal = Arc<brushable_block::BrushableBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CalibratedSculkSensorBlockEntity {
            type Internal = Arc<calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CampfireBlockEntity {
            type Internal = Arc<campfire::CampfireBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ChestBlockEntity {
            type Internal = Arc<chest::ChestBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ChiseledBookshelfBlockEntity {
            type Internal = Arc<chiseled_bookshelf::ChiseledBookshelfBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CommandBlockEntity {
            type Internal = Arc<command_block::CommandBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ComparatorBlockEntity {
            type Internal = Arc<comparator::ComparatorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ConduitBlockEntity {
            type Internal = Arc<conduit::ConduitBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CopperGolemStatueBlockEntity {
            type Internal = Arc<copper_golem_statue::CopperGolemStatueBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CrafterBlockEntity {
            type Internal = Arc<crafter::CrafterBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CreakingHeartBlockEntity {
            type Internal = Arc<creaking_heart::CreakingHeartBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DaylightDetectorBlockEntity {
            type Internal = Arc<daylight_detector::DaylightDetectorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DecoratedPotBlockEntity {
            type Internal = Arc<decorated_pot::DecoratedPotBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DispenserBlockEntity {
            type Internal = Arc<dispenser::DispenserBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DropperBlockEntity {
            type Internal = Arc<dropper::DropperBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EnchantingTableBlockEntity {
            type Internal = Arc<enchanting_table::EnchantingTableBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EnderChestBlockEntity {
            type Internal = Arc<ender_chest::EnderChestBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EndGatewayBlockEntity {
            type Internal = Arc<end_gateway::EndGatewayBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EndPortalBlockEntity {
            type Internal = Arc<end_portal::EndPortalBlockEntity>;
        }
        impl FromResource for plugin::block_entity::FurnaceBlockEntity {
            type Internal = Arc<furnace::FurnaceBlockEntity>;
        }
        impl FromResource for plugin::block_entity::HangingSignBlockEntity {
            type Internal = Arc<hanging_sign::HangingSignBlockEntity>;
        }
        impl FromResource for plugin::block_entity::HopperBlockEntity {
            type Internal = Arc<hopper::HopperBlockEntity>;
        }
        impl FromResource for plugin::block_entity::JigsawBlockEntity {
            type Internal = Arc<jigsaw_block::JigsawBlockEntity>;
        }
        impl FromResource for plugin::block_entity::JukeboxBlockEntity {
            type Internal = Arc<jukebox::JukeboxBlockEntity>;
        }
        impl FromResource for plugin::block_entity::LecternBlockEntity {
            type Internal = Arc<lectern::LecternBlockEntity>;
        }
        impl FromResource for plugin::block_entity::MapBlockEntity {
            type Internal = Arc<map::MapBlockEntity>;
        }
        impl FromResource for plugin::block_entity::MobSpawnerBlockEntity {
            type Internal = Arc<mob_spawner::MobSpawnerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::PistonBlockEntity {
            type Internal = Arc<piston::PistonBlockEntity>;
        }
        impl FromResource for plugin::block_entity::PotentSulfurBlockEntity {
            type Internal = Arc<potent_sulfur::PotentSulfurBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SculkCatalystBlockEntity {
            type Internal = Arc<sculk_catalyst::SculkCatalystBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SculkSensorBlockEntity {
            type Internal = Arc<sculk_sensor::SculkSensorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SculkShriekerBlockEntity {
            type Internal = Arc<sculk_shrieker::SculkShriekerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ShelfBlockEntity {
            type Internal = Arc<shelf::ShelfBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ShulkerBoxBlockEntity {
            type Internal = Arc<shulker_box::ShulkerBoxBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SignBlockEntity {
            type Internal = Arc<sign::SignBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SkullBlockEntity {
            type Internal = Arc<skull::SkullBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SmokerBlockEntity {
            type Internal = Arc<smoker::SmokerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::StructureBlockBlockEntity {
            type Internal = Arc<structure_block::StructureBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TestBlockBlockEntity {
            type Internal = Arc<test_block::TestBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TestInstanceBlockBlockEntity {
            type Internal = Arc<test_instance_block::TestInstanceBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TrappedChestBlockEntity {
            type Internal = Arc<trapped_chest::TrappedChestBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TrialSpawnerBlockEntity {
            type Internal = Arc<trial_spawner::TrialSpawnerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::VaultBlockEntity {
            type Internal = Arc<vault::VaultBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ContainerBlockEntity {
            type Internal = ContainerBlockEntity;
        }
    }

    impl FromResource for plugin::inventory::Inventory {
        type Internal = InventoryProvider;
    }
    impl FromResource for plugin::inventory::PlayerInventory {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::entity::LivingEntity {
        type Internal = Arc<dyn EntityBase>;
    }
    impl FromResource for plugin::entity::Mob {
        type Internal = Arc<dyn EntityBase>;
    }

    impl FromResource for plugin::display::DisplayEntity {
        type Internal = Arc<dyn EntityBase>;
    }
    impl FromResource for plugin::display::BlockDisplayEntity {
        type Internal = Arc<pumpkin_core::entity::decoration::display::BlockDisplayEntity>;
    }
    impl FromResource for plugin::display::ItemDisplayEntity {
        type Internal = Arc<pumpkin_core::entity::decoration::display::ItemDisplayEntity>;
    }
    impl FromResource for plugin::display::TextDisplayEntity {
        type Internal = Arc<pumpkin_core::entity::decoration::display::TextDisplayEntity>;
    }
    impl FromResource for plugin::display::InteractionEntity {
        type Internal = Arc<pumpkin_core::entity::interaction::InteractionEntity>;
    }

    impl FromResource for plugin::world::ChunkBuffer {
        type Internal = ChunkBuffer;
    }

    impl FromResource for plugin::event::Event {
        type Internal = Option<QueuedEvent>;
    }
}

impl pumpkin::plugin::java_packets::Host for PluginHostState {}
impl pumpkin::plugin::bedrock_packets::Host for PluginHostState {}
impl pumpkin::plugin::data_components::Host for PluginHostState {}
impl pumpkin::plugin::enchantments::Host for PluginHostState {}
impl pumpkin::plugin::biomes::Host for PluginHostState {}
impl pumpkin::plugin::attributes::Host for PluginHostState {}
impl pumpkin::plugin::advancement::Host for PluginHostState {}
impl pumpkin::plugin::damage_types::Host for PluginHostState {}
impl pumpkin::plugin::screens::Host for PluginHostState {}
impl pumpkin::plugin::statistics::Host for PluginHostState {}
impl pumpkin::plugin::game_rules::Host for PluginHostState {}
impl pumpkin::plugin::game_events::Host for PluginHostState {}
impl pumpkin::plugin::potions::Host for PluginHostState {}
impl pumpkin::plugin::entity_statuses::Host for PluginHostState {}

pub fn add_to_linker(linker: &mut Linker<PluginHostState>) -> wasmtime::Result<()> {
    bindings::Plugin::add_to_linker::<_, HasSelf<_>>(linker, |state: &mut PluginHostState| state)?;
    Ok(())
}

pub fn prepare_plugin(
    instance_pre: &InstancePre<PluginHostState>,
) -> wasmtime::Result<PluginPre<PluginHostState>> {
    PluginPre::new(instance_pre.clone())
}

/// Loads a native (`cdylib`) WIT v0.2 plugin library and allocates its backing Wasmtime `Store`.
///
/// The caller must start the returned `Store` in `StoreMode::Concurrent`, attach a
/// [`pumpkin_native_wit::ConcurrentStoreRunner`] via [`Plugin::attach_concurrent_store`], and then
/// invoke [`Plugin::call_init_plugin`] and [`Plugin::call_get_metadata`].
pub fn load_native_plugin(
    engine: &Engine,
    path: &Path,
) -> Result<(Plugin, Store<PluginHostState>), PluginInitError> {
    let store = Store::new(engine, PluginHostState::new());
    // SAFETY: Native WIT shared libraries are explicitly placed in the server's `plugins/`
    // directory by the server operator and validated against the host WIT world metadata.
    let plugin = unsafe { bindings::native::Plugin::new(path) }
        .map_err(PluginInitError::InstantiationFailed)?;
    Ok((Plugin::Native(Arc::new(plugin)), store))
}

pub async fn init_plugin(
    engine: &Engine,
    plugin_pre: PluginPre<PluginHostState>,
    legacy_sync_reentry: &LegacySyncReentry,
) -> Result<(Plugin, Store<PluginHostState>, PluginMetadata), PluginInitError> {
    let mut store = Store::new(engine, PluginHostState::new());
    store.limiter(|state| &mut state.limits);
    let plugin = legacy_sync_reentry
        .scope_bootstrap(plugin_pre.instantiate_async(&mut store))
        .await
        .map_err(PluginInitError::InstantiationFailed)?;

    store
        .run_concurrent(async |accessor| {
            legacy_sync_reentry
                .scope_bootstrap(plugin.call_init_plugin(accessor))
                .await
        })
        .await
        .map_err(PluginInitError::CallInitPluginFailed)?
        .map_err(PluginInitError::CallInitPluginFailed)?;

    let metadata = store
        .run_concurrent(async |accessor| {
            legacy_sync_reentry
                .scope_bootstrap(plugin.pumpkin_plugin_metadata().call_get_metadata(accessor))
                .await
        })
        .await
        .map_err(PluginInitError::CallGetMetadataFailed)?
        .map_err(PluginInitError::CallGetMetadataFailed)?;

    let metadata = PluginMetadata {
        name: metadata.name,
        version: metadata.version,
        authors: metadata.authors,
        description: metadata.description,
        dependencies: metadata.dependencies,
        permissions: metadata.permissions,
    };

    store
        .data_mut()
        .permissions
        .clone_from(&metadata.permissions);

    Ok((Plugin::Wasm(Box::new(plugin)), store, metadata))
}

#[cfg(test)]
mod tests {
    use crate::generated_packets::{clientbound_java_any_to_wit, serialize_java_packet};
    use crate::pumpkin::plugin::java_packets::{CSetBlockDestroyStage, ClientboundPacket};
    use pumpkin_protocol::VarInt;
    use pumpkin_protocol::java::client::play::CSetBlockDestroyStage as ProtocolCSetBlockDestroyStage;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::version::JavaMinecraftVersion;

    #[test]
    fn serializes_destroy_stage() {
        for destroy_stage in -1..=9 {
            let packet = ClientboundPacket::CSetBlockDestroyStage(CSetBlockDestroyStage {
                entity_id: 1,
                location: (0, 64, 0),
                destroy_stage,
            });

            let bytes = serialize_java_packet(&packet, JavaMinecraftVersion::V_26_3);
            assert_eq!(
                bytes.as_ref().and_then(|bytes| bytes.last()),
                Some(&(destroy_stage as u8))
            );
        }
    }

    #[test]
    fn converts_negative_destroy_stage_to_wit() {
        let packet = ProtocolCSetBlockDestroyStage {
            entity_id: VarInt(1),
            location: BlockPos::new(0, 64, 0),
            destroy_stage: -1,
        };

        assert!(matches!(
            clientbound_java_any_to_wit(&packet),
            Some(ClientboundPacket::CSetBlockDestroyStage(
                CSetBlockDestroyStage {
                    destroy_stage: -1,
                    ..
                }
            ))
        ));
    }
}
