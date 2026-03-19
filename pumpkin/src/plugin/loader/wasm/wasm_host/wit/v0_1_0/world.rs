use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{PluginHostState, WorldResource},
    wit::v0_1_0::pumpkin::{self, plugin::world::World},
};

impl DowncastResourceExt<WorldResource> for Resource<World> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a WorldResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid world resource handle")
            .downcast_ref::<WorldResource>()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut WorldResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid world resource handle")
            .downcast_mut::<WorldResource>()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> WorldResource {
        state
            .resource_table
            .delete::<WorldResource>(Resource::new_own(self.rep()))
            .expect("invalid world resource handle")
    }
}

impl pumpkin::plugin::world::Host for PluginHostState {}

impl pumpkin::plugin::world::HostWorld for PluginHostState {
    async fn get_id(&mut self, world: Resource<World>) -> String {
        world
            .downcast_ref(self)
            .provider
            .get_world_name()
            .to_string()
    }

    async fn drop(&mut self, rep: Resource<World>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<WorldResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
