use wasmtime::component::Resource;

use crate::{
    entity::player::TitleMode,
    net::DisconnectReason,
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt,
        state::{PlayerResource, PluginHostState, TextComponentResource, WorldResource},
        wit::v0_1_0::{
            events::{
                from_wasm_game_mode, from_wasm_position, to_wasm_game_mode, to_wasm_position,
            },
            pumpkin::{self, plugin::player::Player},
        },
    },
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

    async fn get_position(
        &mut self,
        player: Resource<Player>,
    ) -> pumpkin::plugin::common::Position {
        let position = player.downcast_ref(self).provider.position();
        to_wasm_position(position)
    }

    async fn get_rotation(&mut self, player: Resource<Player>) -> (f32, f32) {
        player.downcast_ref(self).provider.rotation()
    }

    async fn get_world(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::component::Resource<pumpkin::plugin::world::World> {
        let world = player.downcast_ref(self).provider.world();
        self.add_world(world).expect("failed to add world resource")
    }

    async fn get_gamemode(
        &mut self,
        player: Resource<Player>,
    ) -> pumpkin::plugin::common::GameMode {
        let gamemode = player.downcast_ref(self).provider.gamemode.load();
        to_wasm_game_mode(gamemode)
    }

    async fn set_gamemode(
        &mut self,
        player: Resource<Player>,
        mode: pumpkin::plugin::common::GameMode,
    ) -> bool {
        let player = player.downcast_ref(self).provider.clone();
        let mode = from_wasm_game_mode(mode);
        player.set_gamemode(mode).await
    }

    async fn send_system_message(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) {
        let text_resource = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))
            .expect("invalid text-component resource handle");
        let component = text_resource.provider.clone();

        let player = player.downcast_ref(self).provider.clone();
        player.send_system_message_raw(&component, overlay).await;
    }

    async fn show_title(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) {
        let text_resource = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))
            .expect("invalid text-component resource handle");
        let component = text_resource.provider.clone();

        let player = player.downcast_ref(self).provider.clone();
        player.show_title(&component, &TitleMode::Title).await;
    }

    async fn show_subtitle(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) {
        let text_resource = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))
            .expect("invalid text-component resource handle");
        let component = text_resource.provider.clone();

        let player = player.downcast_ref(self).provider.clone();
        player.show_title(&component, &TitleMode::SubTitle).await;
    }

    async fn show_actionbar(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) {
        let text_resource = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))
            .expect("invalid text-component resource handle");
        let component = text_resource.provider.clone();

        let player = player.downcast_ref(self).provider.clone();
        player.show_title(&component, &TitleMode::ActionBar).await;
    }

    async fn send_title_animation(
        &mut self,
        player: Resource<Player>,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    ) {
        let player = player.downcast_ref(self).provider.clone();
        player.send_title_animation(fade_in, stay, fade_out).await;
    }

    async fn teleport(
        &mut self,
        player: Resource<Player>,
        position: pumpkin::plugin::common::Position,
        yaw: f32,
        pitch: f32,
    ) {
        let player = player.downcast_ref(self).provider.clone();
        let position = from_wasm_position(position);
        player.request_teleport(position, yaw, pitch).await;
    }

    async fn teleport_world(
        &mut self,
        player: Resource<Player>,
        world: wasmtime::component::Resource<pumpkin::plugin::world::World>,
        position: pumpkin::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) {
        let world_resource = self
            .resource_table
            .get::<WorldResource>(&Resource::new_own(world.rep()))
            .expect("invalid world resource handle");
        let world = world_resource.provider.clone();

        let player = player.downcast_ref(self).provider.clone();
        let position = from_wasm_position(position);
        player.teleport_world(world, position, yaw, pitch).await;
    }

    async fn kick(
        &mut self,
        player: Resource<Player>,
        message: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) {
        let text_resource = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(message.rep()))
            .expect("invalid text-component resource handle");
        let component = text_resource.provider.clone();

        let player = player.downcast_ref(self).provider.clone();
        player.kick(DisconnectReason::Kicked, component).await;
    }

    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
