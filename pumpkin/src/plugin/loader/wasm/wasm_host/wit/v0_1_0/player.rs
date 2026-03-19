use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{PlayerResource, PluginHostState},
    wit::v0_1_0::pumpkin::{self, plugin::player::Player},
};

impl DowncastResourceExt<PlayerResource> for Resource<Player> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a PlayerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid player resource handle")
            .downcast_ref::<PlayerResource>()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut PlayerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid player resource handle")
            .downcast_mut::<PlayerResource>()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> PlayerResource {
        state
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(self.rep()))
            .expect("invalid player resource handle")
    }
}

impl pumpkin::plugin::player::Host for PluginHostState {}
impl pumpkin::plugin::player::HostPlayer for PluginHostState {
    async fn get_id(&mut self, player: Resource<Player>) -> String {
        player
            .downcast_ref(self)
            .provider
            .gameprofile
            .id
            .to_string()
    }

    async fn get_name(&mut self, player: Resource<Player>) -> String {
        player.downcast_ref(self).provider.gameprofile.name.clone()
    }

    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
