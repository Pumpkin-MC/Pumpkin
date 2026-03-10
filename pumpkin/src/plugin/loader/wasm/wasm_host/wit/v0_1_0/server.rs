use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{PluginHostState, ServerResource},
    wit::v0_1_0::pumpkin::{
        self,
        plugin::server::{Difficulty, Server},
    },
};

impl DowncastResourceExt<ServerResource> for Resource<Server> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a ServerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid server resource handle")
            .downcast_ref::<ServerResource>()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut ServerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid server resource handle")
            .downcast_mut::<ServerResource>()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> ServerResource {
        state
            .resource_table
            .delete::<ServerResource>(Resource::new_own(self.rep()))
            .expect("invalid server resource handle")
    }
}

impl pumpkin::plugin::server::Host for PluginHostState {}

impl pumpkin::plugin::server::HostServer for PluginHostState {
    async fn drop(&mut self, rep: Resource<Server>) -> wasmtime::Result<()> {
        rep.consume(self);
        Ok(())
    }

    async fn get_difficulty(&mut self, server: Resource<Server>) -> Difficulty {
        match server.downcast_ref(self).provider.get_difficulty() {
            pumpkin_util::Difficulty::Peaceful => Difficulty::Peaceful,
            pumpkin_util::Difficulty::Easy => Difficulty::Easy,
            pumpkin_util::Difficulty::Normal => Difficulty::Normal,
            pumpkin_util::Difficulty::Hard => Difficulty::Hard,
        }
    }

    async fn set_difficulty(&mut self, server: Resource<Server>, difficulty: Difficulty) {
        server
            .downcast_ref(self)
            .provider
            .set_difficulty(
                match difficulty {
                    Difficulty::Peaceful => pumpkin_util::Difficulty::Peaceful,
                    Difficulty::Easy => pumpkin_util::Difficulty::Easy,
                    Difficulty::Normal => pumpkin_util::Difficulty::Normal,
                    Difficulty::Hard => pumpkin_util::Difficulty::Hard,
                },
                false,
            )
            .await;
    }

    async fn get_motd(&mut self, server: Resource<Server>) -> String {
        server.downcast_ref(self).provider.get_motd().await
    }

    async fn set_motd(&mut self, server: Resource<Server>, motd: String) {
        server.downcast_ref(self).provider.set_motd(motd).await;
    }
}
