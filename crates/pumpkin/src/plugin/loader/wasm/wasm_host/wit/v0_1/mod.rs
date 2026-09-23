use crate::plugin::{
    PluginMetadata,
    loader::wasm::wasm_host::{
        PluginInitError, PluginInstance, concurrent_store::LegacySyncReentry,
        state::PluginHostState,
    },
};
use pumpkin_host_bindings::PluginPre;
use wasmtime::component::{HasSelf, InstancePre, Linker};
use wasmtime::{Engine, Store};

pub mod advancement;
// wasmtime's `bindgen!` requires every Host trait method to be `async fn`, even the ones whose
// implementation here happens not to need to `.await` anything - so `unused_async_trait_impl`
// can't be avoided without breaking the generated trait signatures.
#[allow(clippy::unused_async_trait_impl)]
pub mod block_entity;
#[allow(clippy::unused_async_trait_impl)]
pub mod boss_bar;
#[allow(clippy::unused_async_trait_impl)]
pub mod commands;
pub mod common;
#[allow(clippy::unused_async_trait_impl)]
pub mod context;
#[allow(clippy::unused_async_trait_impl)]
pub mod datapack;
#[allow(clippy::unused_async_trait_impl)]
pub mod display;
#[allow(clippy::unused_async_trait_impl)]
pub mod enchantment;
#[allow(clippy::unused_async_trait_impl)]
pub mod entity;
pub mod events;
pub mod forms;
pub mod generated_packets;
#[allow(clippy::unused_async_trait_impl)]
pub mod gui;
#[allow(clippy::unused_async_trait_impl)]
pub mod i18n;
#[allow(clippy::unused_async_trait_impl)]
pub mod inventory;
pub mod ipc;
#[allow(clippy::unused_async_trait_impl)]
pub mod item_stack;
pub mod java_dialogs;
#[allow(clippy::unused_async_trait_impl)]
pub mod living_entity;
#[allow(clippy::unused_async_trait_impl)]
pub mod logging;
#[allow(clippy::unused_async_trait_impl)]
pub mod mob;
pub mod permission;
#[allow(clippy::unused_async_trait_impl)]
pub mod player;
#[allow(clippy::unused_async_trait_impl)]
pub mod recipe;
pub mod scheduler;
#[allow(clippy::unused_async_trait_impl)]
pub mod scoreboard;
#[allow(clippy::unused_async_trait_impl)]
pub mod server;
pub mod status_effect;
#[allow(clippy::unused_async_trait_impl)]
pub mod text;
#[allow(clippy::unused_async_trait_impl)]
pub mod uuid;
#[allow(clippy::unused_async_trait_impl)]
pub mod world;

pub use pumpkin_host_bindings::{Plugin, pumpkin};

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
    Plugin::add_to_linker::<_, HasSelf<_>>(linker, |state: &mut PluginHostState| state)?;
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
) -> Result<(PluginInstance, Store<PluginHostState>, PluginMetadata), PluginInitError> {
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

    Ok((PluginInstance::V0_1(plugin), store, metadata))
}
