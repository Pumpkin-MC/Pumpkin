use std::sync::Arc;

use pumpkin_util::Difficulty;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::wasm_host::wit::v0_1_0::pumpkin::plugin::{context::Context, server::Server};

pub trait ServerProvider: Send + Sync {
    fn get_difficulty(&self) -> Difficulty;
}

pub trait ContextProvider: Send + Sync {
    fn get_server(&self) -> Arc<dyn ServerProvider>;
}

pub struct ServerResource {
    pub provider: Arc<dyn ServerProvider>,
}

pub struct ContextResource {
    pub provider: Arc<dyn ContextProvider>,
}

pub struct PluginHostState {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

impl Default for PluginHostState {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginHostState {
    #[must_use]
    pub fn new() -> Self {
        let resource_table = ResourceTable::new();
        Self {
            wasi_ctx: WasiCtxBuilder::new().build(),
            resource_table,
        }
    }

    pub fn add_server(
        &mut self,
        provider: Arc<dyn ServerProvider>,
    ) -> wasmtime::Result<wasmtime::component::Resource<Server>> {
        let resource = self.resource_table.push(ServerResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }

    pub fn add_context(
        &mut self,
        provider: Arc<dyn ContextProvider>,
    ) -> wasmtime::Result<wasmtime::component::Resource<Context>> {
        let resource = self.resource_table.push(ContextResource { provider })?;
        Ok(wasmtime::component::Resource::new_own(resource.rep()))
    }
}

impl WasiView for PluginHostState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi_ctx,
            table: &mut self.resource_table,
        }
    }
}
