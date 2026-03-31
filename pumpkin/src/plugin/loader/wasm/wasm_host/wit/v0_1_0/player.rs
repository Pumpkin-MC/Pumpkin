use std::sync::atomic::Ordering;
use wasmtime::component::Resource;

use crate::{
    entity::{EntityBase, player::TitleMode},
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
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::permission::PermissionLvl;

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

fn to_wit_item_stack(
    stack: &pumpkin_data::item_stack::ItemStack,
) -> Option<pumpkin::plugin::common::ItemStack> {
    if stack.item_count == 0 {
        return None;
    }

    Some(pumpkin::plugin::common::ItemStack {
        registry_key: stack.item.registry_key.to_string(),
        count: stack.item_count,
    })
}

const fn to_wit_permission_level(
    level: PermissionLvl,
) -> pumpkin::plugin::permission::PermissionLevel {
    match level {
        PermissionLvl::Zero => pumpkin::plugin::permission::PermissionLevel::Zero,
        PermissionLvl::One => pumpkin::plugin::permission::PermissionLevel::One,
        PermissionLvl::Two => pumpkin::plugin::permission::PermissionLevel::Two,
        PermissionLvl::Three => pumpkin::plugin::permission::PermissionLevel::Three,
        PermissionLvl::Four => pumpkin::plugin::permission::PermissionLevel::Four,
    }
}

const fn from_wit_permission_level(
    level: pumpkin::plugin::permission::PermissionLevel,
) -> PermissionLvl {
    match level {
        pumpkin::plugin::permission::PermissionLevel::Zero => PermissionLvl::Zero,
        pumpkin::plugin::permission::PermissionLevel::One => PermissionLvl::One,
        pumpkin::plugin::permission::PermissionLevel::Two => PermissionLvl::Two,
        pumpkin::plugin::permission::PermissionLevel::Three => PermissionLvl::Three,
        pumpkin::plugin::permission::PermissionLevel::Four => PermissionLvl::Four,
    }
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

    async fn get_locale(&mut self, player: Resource<Player>) -> Result<String, String> {
        let player = player_from_resource(self, &player)?;
        let config = player.config.load();
        Ok(config.locale.clone())
    }

    async fn get_ping(&mut self, player: Resource<Player>) -> Result<u32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.ping.load(Ordering::Relaxed))
    }

    async fn get_permission_level(
        &mut self,
        player: Resource<Player>,
    ) -> Result<pumpkin::plugin::permission::PermissionLevel, String> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wit_permission_level(player.permission_lvl.load()))
    }

    async fn set_permission_level(
        &mut self,
        player: Resource<Player>,
        level: pumpkin::plugin::permission::PermissionLevel,
    ) -> Result<(), String> {
        let player = player_from_resource(self, &player)?;
        let Some(server) = self.server.as_ref() else {
            return Err("server not available".to_string());
        };
        let level = from_wit_permission_level(level);
        let command_dispatcher = server.command_dispatcher.read().await;
        player
            .set_permission_lvl(server, level, &command_dispatcher)
            .await;
        Ok(())
    }

    async fn has_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> Result<bool, String> {
        let player = player_from_resource(self, &player)?;
        let Some(server) = self.server.as_ref() else {
            return Err("server not available".to_string());
        };
        Ok(player.has_permission(server, &node).await)
    }

    async fn get_display_name(
        &mut self,
        player: Resource<Player>,
    ) -> Result<Resource<pumpkin::plugin::text::TextComponent>, String> {
        let player = player_from_resource(self, &player)?;
        let display_name = player.get_display_name().await;
        self.add_text_component(display_name)
            .map_err(|_| "failed to add text-component resource".to_string())
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

    async fn get_selected_slot(&mut self, player: Resource<Player>) -> Result<u8, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.inventory.get_selected_slot())
    }

    async fn get_item_in_hand(
        &mut self,
        player: Resource<Player>,
        hand: pumpkin::plugin::common::Hand,
    ) -> Result<Option<pumpkin::plugin::common::ItemStack>, String> {
        let player = player_from_resource(self, &player)?;
        let inventory = player.inventory();
        let item_stack = match hand {
            pumpkin::plugin::common::Hand::Left => inventory.off_hand_item().await,
            pumpkin::plugin::common::Hand::Right => inventory.held_item(),
        };
        let item_stack = item_stack.lock().await.clone();
        Ok(to_wit_item_stack(&item_stack))
    }

    async fn get_inventory_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
    ) -> Result<Option<pumpkin::plugin::common::ItemStack>, String> {
        let player = player_from_resource(self, &player)?;
        let slot = slot as usize;
        if slot >= PlayerInventory::MAIN_SIZE {
            return Err("invalid inventory slot".to_string());
        }

        let item_stack = player.inventory.main_inventory[slot].lock().await.clone();
        Ok(to_wit_item_stack(&item_stack))
    }

    async fn get_health(&mut self, player: Resource<Player>) -> Result<f32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.health.load())
    }

    async fn get_max_health(&mut self, player: Resource<Player>) -> Result<f32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.get_max_health())
    }

    async fn get_food_level(&mut self, player: Resource<Player>) -> Result<u8, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.hunger_manager.level.load())
    }

    async fn get_saturation(&mut self, player: Resource<Player>) -> Result<f32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.hunger_manager.saturation.load())
    }

    async fn get_experience_level(&mut self, player: Resource<Player>) -> Result<i32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_level.load(Ordering::Relaxed))
    }

    async fn get_experience_progress(&mut self, player: Resource<Player>) -> Result<f32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_progress.load())
    }

    async fn get_experience_points(&mut self, player: Resource<Player>) -> Result<i32, String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_points.load(Ordering::Relaxed))
    }

    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
