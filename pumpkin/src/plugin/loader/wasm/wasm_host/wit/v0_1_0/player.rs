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

fn player_from_resource(
    state: &PluginHostState,
    player: &Resource<Player>,
) -> Result<std::sync::Arc<crate::entity::player::Player>, String> {
    state
        .resource_table
        .get::<PlayerResource>(&Resource::new_own(player.rep()))
        .map_err(|_| "invalid player resource handle".to_string())
        .map(|resource| resource.provider.clone())
}

fn text_component_from_resource(
    state: &PluginHostState,
    text: &Resource<pumpkin::plugin::text::TextComponent>,
) -> Result<pumpkin_util::text::TextComponent, String> {
    state
        .resource_table
        .get::<TextComponentResource>(&Resource::new_own(text.rep()))
        .map_err(|_| "invalid text-component resource handle".to_string())
        .map(|resource| resource.provider.clone())
}

fn world_from_resource(
    state: &PluginHostState,
    world: &Resource<pumpkin::plugin::world::World>,
) -> Result<std::sync::Arc<crate::world::World>, String> {
    state
        .resource_table
        .get::<WorldResource>(&Resource::new_own(world.rep()))
        .map_err(|_| "invalid world resource handle".to_string())
        .map(|resource| resource.provider.clone())
}

impl DowncastResourceExt<PlayerResource> for Resource<Player> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a PlayerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
            .unwrap()
            .downcast_ref::<PlayerResource>()
            .ok_or("resource type mismatch")
            .map_err(wasmtime::Error::msg)
            .unwrap()
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut PlayerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
            .unwrap()
            .downcast_mut::<PlayerResource>()
            .ok_or("resource type mismatch")
            .map_err(wasmtime::Error::msg)
            .unwrap()
    }

    fn consume(self, state: &mut PluginHostState) -> PlayerResource {
        state
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(self.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
            .unwrap()
    }
}

impl pumpkin::plugin::player::Host for PluginHostState {}
impl pumpkin::plugin::player::HostPlayer for PluginHostState {
    async fn get_id(&mut self, player: Resource<Player>) -> Result<String, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.gameprofile.id.to_string())
    }

    async fn get_name(&mut self, player: Resource<Player>) -> Result<String, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.gameprofile.name.clone())
    }

    async fn get_position(
        &mut self,
        player: Resource<Player>,
    ) -> Result<pumpkin::plugin::common::Position, String> {
        let player = player_from_resource(self, &player)?;
        let position = player.position();
        Ok(to_wasm_position(position))
    }

    async fn get_rotation(&mut self, player: Resource<Player>) -> Result<(f32, f32), String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.rotation())
    }

    async fn get_world(
        &mut self,
        player: Resource<Player>,
    ) -> Result<wasmtime::component::Resource<pumpkin::plugin::world::World>, String> {
        let player = player_from_resource(self, &player)?;
        let world = player.world();
        self.add_world(world)
            .map_err(|_| "failed to add world resource".to_string())
    }

    async fn get_gamemode(
        &mut self,
        player: Resource<Player>,
    ) -> Result<pumpkin::plugin::common::GameMode, String> {
        let player = player_from_resource(self, &player)?;
        let gamemode = player.gamemode.load();
        Ok(to_wasm_game_mode(gamemode))
    }

    async fn set_gamemode(
        &mut self,
        player: Resource<Player>,
        mode: pumpkin::plugin::common::GameMode,
    ) -> Result<bool, String> {
        let player = player_from_resource(self, &player)?;
        let mode = from_wasm_game_mode(mode);
        Ok(player.set_gamemode(mode).await)
    }

    async fn send_system_message(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) -> Result<(), String> {
        let component = text_component_from_resource(self, &text)?;
        let player = player_from_resource(self, &player)?;
        player.send_system_message_raw(&component, overlay).await;
        Ok(())
    }

    async fn show_title(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> Result<(), String> {
        let component = text_component_from_resource(self, &text)?;
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::Title).await;
        Ok(())
    }

    async fn show_subtitle(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> Result<(), String> {
        let component = text_component_from_resource(self, &text)?;
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::SubTitle).await;
        Ok(())
    }

    async fn show_actionbar(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> Result<(), String> {
        let component = text_component_from_resource(self, &text)?;
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::ActionBar).await;
        Ok(())
    }

    async fn send_title_animation(
        &mut self,
        player: Resource<Player>,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    ) -> Result<(), String> {
        let player = player_from_resource(self, &player)?;
        player.send_title_animation(fade_in, stay, fade_out).await;
        Ok(())
    }

    async fn teleport(
        &mut self,
        player: Resource<Player>,
        position: pumpkin::plugin::common::Position,
        yaw: f32,
        pitch: f32,
    ) -> Result<(), String> {
        let player = player_from_resource(self, &player)?;
        let position = from_wasm_position(position);
        player.request_teleport(position, yaw, pitch).await;
        Ok(())
    }

    async fn teleport_world(
        &mut self,
        player: Resource<Player>,
        world: wasmtime::component::Resource<pumpkin::plugin::world::World>,
        position: pumpkin::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) -> Result<(), String> {
        let world = world_from_resource(self, &world)?;
        let player = player_from_resource(self, &player)?;
        let position = from_wasm_position(position);
        player.teleport_world(world, position, yaw, pitch).await;
        Ok(())
    }

    async fn kick(
        &mut self,
        player: Resource<Player>,
        message: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> Result<(), String> {
        let component = text_component_from_resource(self, &message)?;
        let player = player_from_resource(self, &player)?;
        player.kick(DisconnectReason::Kicked, component).await;
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
