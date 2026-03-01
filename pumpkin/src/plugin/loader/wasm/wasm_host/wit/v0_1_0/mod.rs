use crate::plugin::{
    PluginMetadata,
    loader::wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState},
};
use tokio::sync::Mutex;
use wasmtime::component::{Component, HasData, Linker, bindgen};
use wasmtime::{Engine, Store};

pub mod commands;
pub mod common;
pub mod context;
pub mod entity;
pub mod events;
pub mod logging;
pub mod player;
pub mod server;
pub mod text;
pub mod world;

bindgen!({
    path: "../pumpkin-plugin-wit/v0.1.0",
    world: "plugin",
    imports: { default: async },
    exports: { default: async },
});

struct PluginHostComponent;

impl HasData for PluginHostComponent {
    type Data<'a> = &'a mut PluginHostState;
}

pub fn setup_linker(engine: &Engine) -> wasmtime::Result<Linker<PluginHostState>> {
    let mut linker = Linker::new(engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    Plugin::add_to_linker::<_, PluginHostComponent>(&mut linker, |state: &mut PluginHostState| {
        state
    })?;
    Ok(linker)
}

pub async fn init_plugin(
    engine: &Engine,
    linker: &Linker<PluginHostState>,
    component: Component,
) -> wasmtime::Result<(WasmPlugin, PluginMetadata)> {
    let mut store = Store::new(engine, PluginHostState::new());
    let plugin = Plugin::instantiate_async(&mut store, &component, linker).await?;

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
    };

    Ok((
        WasmPlugin {
            plugin_instance: PluginInstance::V0_1_0(plugin),
            store: Mutex::new(store),
        },
        metadata,
    ))
}
