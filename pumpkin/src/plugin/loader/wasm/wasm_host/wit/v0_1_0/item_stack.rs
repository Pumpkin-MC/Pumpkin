use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{self, plugin::item_stack::ItemStack},
};

impl pumpkin::plugin::item_stack::Host for PluginHostState {}

impl pumpkin::plugin::item_stack::HostItemStack for PluginHostState {
    async fn drop(&mut self, _rep: Resource<ItemStack>) -> wasmtime::Result<()> {
        todo!()
    }
}
