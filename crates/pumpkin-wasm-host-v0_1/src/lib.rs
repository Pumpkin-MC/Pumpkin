#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(clippy::significant_drop_in_scrutinee)]
// Not warn event sending macros
#![allow(unused_labels, deprecated)]

use std::path::Path;

use crate::bindings::PluginPre;
use pumpkin_core::plugin::PluginMetadata;
use pumpkin_wasm_host_common::{
    concurrent_store::{LegacyGuestScope, LegacySyncReentry},
    plugin::PluginInitError,
    state::PluginHostState,
};
use wasmtime::component::{HasSelf, InstancePre, Linker, Resource};
use wasmtime::{AsContextMut, Engine, Store};

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

/// Loaded v0.1 guest plugin exports, backed by either a WebAssembly component or a native `cdylib`.
pub enum Plugin {
    Wasm(Box<bindings::Plugin>),
    Native(bindings::native::Plugin),
}

impl Plugin {
    #[must_use]
    pub const fn is_native(&self) -> bool {
        matches!(self, Self::Native(_))
    }

    #[must_use]
    pub fn native_library(
        &self,
    ) -> Option<std::sync::Arc<pumpkin_native_wit::libloading::Library>> {
        match self {
            Self::Wasm(_) => None,
            Self::Native(plugin) => Some(plugin.library()),
        }
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
            Self::Native(plugin) => guest.call_native(|store| plugin.call_on_load(store, context)),
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
                guest.call_native(|store| plugin.call_on_unload(store, context))
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
                guest.call_native(|store| plugin.call_handle_ipc_message(store, &sender, &message))
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
                guest.call_native(|store| plugin.call_handle_task(store, handler_id, server))
            }
        }
    }

    pub async fn call_handle_event(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        event_id: u32,
        server: Resource<pumpkin::plugin::server::Server>,
        event: pumpkin::plugin::event::Event,
    ) -> wasmtime::Result<pumpkin::plugin::event::Event> {
        match self {
            Self::Wasm(plugin) => {
                let function = *plugin.func_handle_event().func();
                guest
                    .call_func::<(
                        u32,
                        Resource<pumpkin::plugin::server::Server>,
                        pumpkin::plugin::event::Event,
                    ), (pumpkin::plugin::event::Event,)>(
                        function, (event_id, server, event)
                    )
                    .await
                    .map(|(returned,)| returned)
            }
            Self::Native(plugin) => {
                guest.call_native(|store| plugin.call_handle_event(store, event_id, server, &event))
            }
        }
    }

    pub async fn call_handle_command(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        command_id: u32,
        sender: Resource<pumpkin::plugin::command::CommandSender>,
        server: Resource<pumpkin::plugin::server::Server>,
        args: Resource<pumpkin::plugin::command::ConsumedArgs>,
    ) -> wasmtime::Result<Result<i32, pumpkin::plugin::command::CommandError>> {
        match self {
            Self::Wasm(plugin) => guest
                .call(
                    plugin.func_handle_command(),
                    (command_id, sender, server, args),
                )
                .await
                .map(|(result,)| result),
            Self::Native(plugin) => guest.call_native(|store| {
                plugin.call_handle_command(store, command_id, sender, server, args)
            }),
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
            Self::Wasm(plugin) => {
                let function = *plugin.func_handle_command_suggestion().func();
                guest
                    .call_func::<(
                        u32,
                        Resource<pumpkin::plugin::command::CommandSender>,
                        Resource<pumpkin::plugin::server::Server>,
                        pumpkin::plugin::command::SuggestionRequest,
                    ), (pumpkin::plugin::command::CommandSuggestions,)>(
                        function,
                        (handler_id, sender, server, request),
                    )
                    .await
                    .map(|(response,)| response)
            }
            Self::Native(plugin) => guest.call_native(|store| {
                plugin.call_handle_command_suggestion(store, handler_id, sender, server, &request)
            }),
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
            Self::Native(plugin) => guest.call_native(|store| {
                plugin.call_handle_ai_goal_start(store, goal_id, server, entity)
            }),
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
            Self::Native(plugin) => guest.call_native(|store| {
                plugin.call_handle_ai_goal_tick(store, goal_id, server, entity)
            }),
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
            Self::Native(plugin) => guest.call_native(|store| {
                plugin.call_handle_ai_goal_stop(store, goal_id, server, entity)
            }),
        }
    }

    pub async fn call_handle_generate_phase(
        &self,
        guest: &mut LegacyGuestScope<'_, PluginHostState>,
        generator_id: u32,
        phase: pumpkin::plugin::world::GenerationPhase,
        chunk: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<()> {
        match self {
            Self::Wasm(plugin) => {
                guest
                    .call(
                        plugin.func_handle_generate_phase(),
                        (generator_id, phase, chunk),
                    )
                    .await
            }
            Self::Native(plugin) => guest.call_native(|store| {
                plugin.call_handle_generate_phase(store, generator_id, phase, chunk)
            }),
        }
    }
}

mod resource_with {
    use super::pumpkin::plugin;
    use pumpkin_core::{
        command::CommandSender,
        entity::{EntityBase, player::Player},
        plugin::{Context, api::gui::PluginGui},
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

    legacy_sync_reentry
        .scope_bootstrap(plugin.call_init_plugin(&mut store))
        .await
        .map_err(PluginInitError::CallInitPluginFailed)?;

    let metadata = legacy_sync_reentry
        .scope_bootstrap(
            plugin
                .pumpkin_plugin_metadata()
                .call_get_metadata(&mut store),
        )
        .await
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

/// Loads and initializes a native (`cdylib`) plugin implementing the v0.1 WIT world.
///
/// Loading a native dynamic library executes its OS-level initializers inside the host process,
/// so callers should only pass trusted plugin libraries configured by the server administrator.
pub async fn init_native_plugin(
    engine: &Engine,
    path: &Path,
    legacy_sync_reentry: &LegacySyncReentry,
) -> Result<(Plugin, Store<PluginHostState>, PluginMetadata), PluginInitError> {
    // SAFETY: Caller vouches that `path` is a trusted native plugin configured by the server
    // administrator; `bindings::native::Plugin::new` verifies the exported WIT metadata and all
    // required symbols before returning.
    let plugin =
        unsafe { bindings::native::Plugin::new(path) }.map_err(PluginInitError::ApiMismatch)?;

    let mut store = Store::new(engine, PluginHostState::new());
    store.limiter(|state| &mut state.limits);

    legacy_sync_reentry
        .scope_bootstrap(async { plugin.call_init_plugin(store.as_context_mut()) })
        .await
        .map_err(PluginInitError::CallInitPluginFailed)?;

    let metadata = legacy_sync_reentry
        .scope_bootstrap(async { plugin.call_get_metadata(store.as_context_mut()) })
        .await
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

    Ok((Plugin::Native(plugin), store, metadata))
}
