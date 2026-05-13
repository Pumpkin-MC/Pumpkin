use crate::plugin::{
    PluginMetadata,
    loader::wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState},
};
use tokio::sync::Mutex;
use wasmtime::component::{HasSelf, InstancePre, Linker, bindgen};
use wasmtime::{Engine, Store};

pub mod block_entity;
pub mod boss_bar;
pub mod commands;
pub mod common;
pub mod context;
pub mod entity;
pub mod events;
pub mod generated_packets;
pub mod gui;
pub mod i18n;
pub mod logging;
pub mod permission;
pub mod player;
pub mod scheduler;
pub mod scoreboard;
pub mod server;
pub mod text;
pub mod world;

bindgen!({
    path: "../pumpkin-plugin-wit/v0.1",
    world: "plugin",
    imports: { default: async | trappable },
    exports: { default: async | trappable},
});

impl pumpkin::plugin::java_packets::Host for PluginHostState {}
impl pumpkin::plugin::bedrock_packets::Host for PluginHostState {}

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
) -> wasmtime::Result<(WasmPlugin, PluginMetadata)> {
    let mut store = Store::new(engine, PluginHostState::new());
    let plugin = plugin_pre.instantiate_async(&mut store).await?;

    plugin.call_init_plugin(&mut store).await?;

    let metadata = plugin
        .pumpkin_plugin_metadata()
        .call_get_metadata(&mut store)
        .await?;

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

    Ok((
        WasmPlugin {
            plugin_instance: PluginInstance::V0_1(plugin),
            store: Mutex::new(store),
        },
        metadata,
    ))
}
