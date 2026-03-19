use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, ServerResource},
    wit::v0_1_0::pumpkin::{
        self,
        plugin::server::{Difficulty, Server},
    },
};

impl pumpkin::plugin::server::Host for PluginHostState {}

impl pumpkin::plugin::server::HostServer for PluginHostState {
    async fn drop(&mut self, rep: Resource<Server>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ServerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }

    async fn get_difficulty(&mut self, server: Resource<Server>) -> Difficulty {
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
