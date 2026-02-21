use tokio::sync::Mutex;
use wasmtime::component::{Component, HasData, Linker, bindgen};
use wasmtime::{Engine, Store};

use crate::wasm_host::state::{ContextResource, ServerResource};
use crate::wasm_host::wit::v0_1_0::pumpkin::plugin::{player, server::{Difficulty, Server}};
use crate::{
    metadata::PluginMetadata,
    wasm_host::{PluginInstance, WasmPlugin, logging::log_tracing, state::PluginHostState},
};

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

impl pumpkin::plugin::logging::Host for PluginHostState {
    async fn log(&mut self, level: pumpkin::plugin::logging::Level, message: String) {
        match level {
            pumpkin::plugin::logging::Level::Trace => tracing::trace!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Debug => tracing::debug!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Info => tracing::info!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Warn => tracing::warn!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Error => tracing::error!("[plugin] {message}"),
        }
    }

    async fn log_tracing(&mut self, event: Vec<u8>) {
        log_tracing(event).await;
    }
}

impl pumpkin::plugin::server::HostServer for PluginHostState {
    async fn drop(&mut self, rep: wasmtime::component::Resource<Server>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ServerResource>(wasmtime::component::Resource::new_own(rep.rep()));
        Ok(())
    }

    async fn get_difficulty(
        &mut self,
        server: wasmtime::component::Resource<Server>,
    ) -> Difficulty {
        let resource: &ServerResource = self
            .resource_table
            .get_any_mut(server.rep())
            .expect("invalid server resource handle")
            .downcast_ref::<ServerResource>()
            .expect("resource type mismatch");

        match resource.provider.get_difficulty() {
            pumpkin_util::Difficulty::Peaceful => Difficulty::Peaceful,
            pumpkin_util::Difficulty::Easy => Difficulty::Easy,
            pumpkin_util::Difficulty::Normal => Difficulty::Normal,
            pumpkin_util::Difficulty::Hard => Difficulty::Hard,
        }
    }
}


impl pumpkin::plugin::context::HostContext for PluginHostState {
    async fn drop(&mut self, rep: wasmtime::component::Resource<Context>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ContextResource>(wasmtime::component::Resource::new_own(rep.rep()));
        Ok(())
    }

    async fn get_server(
        &mut self,
        context: wasmtime::component::Resource<Context>,
    ) -> wasmtime::component::Resource<Server> {
        let resource = self
            .resource_table
            .get_any_mut(context.rep())
            .expect("invalid context resource handle")
            .downcast_ref::<ContextResource>()
            .expect("resource type mismatch");
        let server_provider = resource.provider.get_server();
        self.add_server(server_provider)
            .expect("failed to add server resource")
    }

    async fn register_event(
        &mut self,
        _context: wasmtime::component::Resource<Context>,
        _function_pointer: i32,
    ) {
        todo!()
    }
}

impl pumpkin::plugin::player::HostPlayer for PluginHostState {
    async fn drop(
        &mut self,
        rep: wasmtime::component::Resource<pumpkin::plugin::player::Player>,
    ) -> wasmtime::Result<()> {
        // TODO: implement
        todo!()
    }

    async fn get_id(&mut self, player: wasmtime::component::Resource<player::Player>) -> String { todo!() }
}

impl pumpkin::plugin::common::HostTextComponent for PluginHostState {
    async fn drop(
        &mut self,
        _rep: wasmtime::component::Resource<pumpkin::plugin::common::TextComponent>,
    ) -> wasmtime::Result<()> {
        // TODO: implement
        todo!()
    }
}


impl pumpkin::plugin::player::Host for PluginHostState {}
impl pumpkin::plugin::common::Host for PluginHostState {}
impl pumpkin::plugin::context::Host for PluginHostState {}
impl pumpkin::plugin::server::Host for PluginHostState {}
impl pumpkin::plugin::event::Host for PluginHostState {}

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
            plugin: PluginInstance::V0_1_0(plugin),
            store: Mutex::new(store),
        },
        metadata,
    ))
}
