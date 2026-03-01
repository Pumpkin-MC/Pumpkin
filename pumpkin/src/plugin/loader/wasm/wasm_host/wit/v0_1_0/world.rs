use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{self, plugin::world::World},
};

impl pumpkin::plugin::world::Host for PluginHostState {}

impl pumpkin::plugin::world::HostWorld for PluginHostState {
    async fn get_id(&mut self, world: Resource<World>) -> String {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<World>) -> wasmtime::Result<()> {
        todo!()
    }
}
