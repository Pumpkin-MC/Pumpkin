use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PlayerResource, PluginHostState},
    wit::v0_1_0::pumpkin::{self, plugin::player::Player},
};

impl pumpkin::plugin::player::Host for PluginHostState {}
impl pumpkin::plugin::player::HostPlayer for PluginHostState {
    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }

    async fn get_id(&mut self, player: Resource<Player>) -> String {
        let resource = self
            .resource_table
            .get_any_mut(player.rep())
            .expect("invalid player resource handle")
            .downcast_ref::<PlayerResource>()
            .expect("resource type mismatch");
        resource.provider.gameprofile.id.to_string()
    }
}
