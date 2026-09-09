use crate::plugin::{
    PluginMetadata,
    loader::wasm::wasm_host::{
        PluginInitError, PluginInstance, concurrent_store::LegacySyncReentry,
        state::PluginHostState,
    },
};
use wasmtime::component::{HasSelf, InstancePre, Linker, bindgen};
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

bindgen!({
    path: "../pumpkin-plugin-wit/v0.1",
    world: "plugin",
    imports: {
        "pumpkin:plugin/command@0.1.0.[method]command-sender.has-permission": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.disable-pack": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.enable-pack": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.execute-function": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.reload": async | store | trappable,
        "pumpkin:plugin/ipc@0.1.0.send-ipc-message": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.add-effect": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.add-experience-levels": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.add-experience-points": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.award-advancement": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.award-advancement-criterion": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.ban": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.ban-ip": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.damage": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.has-permission": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.heal": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.kill": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.open-ender-chest": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.open-gui": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.respawn": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-experience-level": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-experience-points": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-experience-progress": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-food-level": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-gamemode": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-permission-level": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.teleport": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.teleport-world": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]java-player.show-dialog": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]java-player.clear-dialog": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]java-player.kick": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]bedrock-player.kick": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]ban-manager.ban-ip": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]ban-manager.ban-player": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]op-manager.deop-player": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]op-manager.op-player": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.broadcast": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.create-world": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.execute-command": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.save-all": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.unload-world": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]whitelist-manager.set-enabled": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.add-passenger": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.eject-passengers": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.remove-passenger": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.set-swimming": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.set-vehicle": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.teleport": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]living-entity.damage": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]mob.clear-ai-goals": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.create-explosion": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.save": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block-by-id": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block-by-name": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block-state": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-raining": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-thundering": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.spawn-entity": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.strike-lightning": async | store | trappable,
        default: async | trappable,
    },
    exports: { default: async | store | trappable},
});

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
