use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::plugin::api::gui::PluginScreenHandler;
use crate::{
    entity::{EntityBase, player::TitleMode},
    net::DisconnectReason,
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt,
        state::{
            GuiResource, PlayerResource, PluginHostState, TextComponentResource, WorldResource,
        },
        wit::v0_1::{
            events::{
                from_wasm_game_mode, from_wasm_position, to_wasm_game_mode, to_wasm_position,
            },
            pumpkin::{
                self,
                plugin::player::{Player, PlayerSkin, SkinParts},
                plugin::world::World,
            },
        },
    },
};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_util::permission::PermissionLvl;

pub fn player_from_resource(
    state: &PluginHostState,
    player: &Resource<Player>,
) -> wasmtime::Result<std::sync::Arc<crate::entity::player::Player>> {
    state
        .resource_table
        .get::<PlayerResource>(&Resource::new_own(player.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
        .map(|resource| resource.provider.clone())
}

pub(crate) fn text_component_from_resource(
    state: &PluginHostState,
    text: &Resource<pumpkin::plugin::text::TextComponent>,
) -> pumpkin_util::text::TextComponent {
    state
        .resource_table
        .get::<TextComponentResource>(&Resource::new_own(text.rep()))
        .expect("invalid text-component resource handle")
        .provider
        .clone()
}

fn world_from_resource(
    state: &PluginHostState,
    world: &Resource<pumpkin::plugin::world::World>,
) -> std::sync::Arc<crate::world::World> {
    state
        .resource_table
        .get::<WorldResource>(&Resource::new_own(world.rep()))
        .expect("invalid world resource handle")
        .provider
        .clone()
}

pub(super) fn to_wit_item_stack(
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
    async fn as_entity(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::world::Entity>> {
        let player = player_from_resource(self, &player)?;
        self.add_entity(player as Arc<dyn EntityBase>)
            .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))
    }

    async fn get_id(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.gameprofile.id.to_string())
    }

    async fn get_name(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.gameprofile.name.clone())
    }

    async fn get_position(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::common::Position> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wasm_position(player.position()))
    }

    async fn get_yaw(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.entity.yaw.load())
    }

    async fn get_pitch(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.entity.pitch.load())
    }

    async fn get_world(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<wasmtime::component::Resource<pumpkin::plugin::world::World>> {
        let player = player_from_resource(self, &player)?;
        let world = player.world();
        self.add_world(world)
            .map_err(|_| wasmtime::Error::msg("failed to add world resource"))
    }

    async fn get_gamemode(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::common::GameMode> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wasm_game_mode(player.gamemode.load()))
    }

    async fn set_gamemode(
        &mut self,
        player: Resource<Player>,
        mode: pumpkin::plugin::common::GameMode,
    ) -> wasmtime::Result<bool> {
        let player = player_from_resource(self, &player)?;
        Ok(player.set_gamemode(from_wasm_game_mode(mode)).await)
    }

    async fn get_locale(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.config.load().locale.clone())
    }

    async fn get_ping(&mut self, player: Resource<Player>) -> wasmtime::Result<u32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.ping.load(Ordering::Relaxed))
    }

    async fn get_permission_level(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::permission::PermissionLevel> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wit_permission_level(player.permission_lvl.load()))
    }

    async fn set_permission_level(
        &mut self,
        player: Resource<Player>,
        level: pumpkin::plugin::permission::PermissionLevel,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        let level = from_wit_permission_level(level);
        let command_dispatcher = server.command_dispatcher.read().await;
        player
            .set_permission_lvl(server, level, &command_dispatcher)
            .await;
        Ok(())
    }

    async fn set_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");

        let mut perm_manager = server.permission_manager.write().await;
        let attachment = perm_manager.get_attachment(player.gameprofile.id);
        drop(perm_manager);

        attachment.write().await.set_permission(&node, value);

        Ok(())
    }

    async fn unset_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");

        let mut perm_manager = server.permission_manager.write().await;
        let attachment = perm_manager.get_attachment(player.gameprofile.id);
        drop(perm_manager);

        attachment.write().await.unset_permission(&node);

        Ok(())
    }

    async fn has_permission_set(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<Option<bool>> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");

        let mut perm_manager = server.permission_manager.write().await;
        let attachment = perm_manager.get_attachment(player.gameprofile.id);
        drop(perm_manager);

        Ok(attachment.read().await.has_permission_set(&node))
    }

    async fn has_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<bool> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        Ok(player.has_permission(server, &node).await)
    }

    async fn get_display_name(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::text::TextComponent>> {
        let player = player_from_resource(self, &player)?;
        let display_name = player.get_display_name().await;
        self.add_text_component(display_name)
            .map_err(|_| wasmtime::Error::msg("failed to add text-component resource"))
    }

    async fn set_display_name(
        &mut self,
        player: Resource<Player>,
        display_name: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let display_name = text_component_from_resource(self, &display_name);
        let player = player_from_resource(self, &player)?;
        player.set_display_name(Some(display_name)).await;
        Ok(())
    }

    async fn get_tab_list_name(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<pumpkin::plugin::text::TextComponent>>> {
        let player = player_from_resource(self, &player)?;
        let tab_list_name = player.get_tab_list_name().await;
        tab_list_name.map_or_else(
            || Ok(None),
            |name| {
                self.add_text_component(name)
                    .map(Some)
                    .map_err(|_| wasmtime::Error::msg("failed to add text-component resource"))
            },
        )
    }

    async fn set_tab_list_name(
        &mut self,
        player: Resource<Player>,
        name: Option<wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let name = name.map(|n| text_component_from_resource(self, &n));
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_name(name).await;
        Ok(())
    }

    async fn send_system_message(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.send_system_message_raw(&component, overlay).await;
        Ok(())
    }

    async fn set_tab_list_header_footer(
        &mut self,
        player: Resource<Player>,
        header: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        footer: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let header = text_component_from_resource(self, &header);
        let footer = text_component_from_resource(self, &footer);
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_header_footer(header, footer).await;
        Ok(())
    }

    async fn set_tab_list_order(
        &mut self,
        player: Resource<Player>,
        order: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_order(order).await;
        Ok(())
    }

    async fn set_tab_list_latency(
        &mut self,
        player: Resource<Player>,
        latency: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_latency(latency).await;
        Ok(())
    }

    async fn set_tab_list_listed(
        &mut self,
        player: Resource<Player>,
        listed: bool,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_listed(listed).await;
        Ok(())
    }

    async fn show_title(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::Title).await;
        Ok(())
    }

    async fn show_subtitle(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::SubTitle).await;
        Ok(())
    }

    async fn show_actionbar(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
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
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.send_title_animation(fade_in, stay, fade_out).await;
        Ok(())
    }

    async fn teleport(
        &mut self,
        player: Resource<Player>,
        position: pumpkin::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Resource<World>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let world = world_from_resource(self, &world);
        player
            .teleport(from_wasm_position(position), yaw, pitch, world)
            .await;
        Ok(())
    }

    async fn teleport_world(
        &mut self,
        player: Resource<Player>,
        world: wasmtime::component::Resource<pumpkin::plugin::world::World>,
        position: pumpkin::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) -> wasmtime::Result<()> {
        let world = world_from_resource(self, &world);
        let player = player_from_resource(self, &player)?;
        player
            .teleport_world(world, from_wasm_position(position), yaw, pitch)
            .await;
        Ok(())
    }

    async fn kick(
        &mut self,
        player: Resource<Player>,
        message: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &message);
        let player = player_from_resource(self, &player)?;
        player.kick(DisconnectReason::Kicked, component).await;
        Ok(())
    }

    async fn respawn(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.respawn().await;
        Ok(())
    }

    async fn open_gui(
        &mut self,
        player: Resource<Player>,
        gui: Resource<pumpkin::plugin::gui::Gui>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let gui_res = self
            .resource_table
            .get::<GuiResource>(&Resource::new_own(gui.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid gui resource handle"))?;
        let gui = gui_res.provider.lock().await;

        player.increment_screen_handler_sync_id();
        let sync_id = player.screen_handler_sync_id.load(Ordering::Relaxed);
        let screen_handler = Arc::new(Mutex::new(PluginScreenHandler::new(
            sync_id,
            gui.window_type,
            &gui.inventory,
            gui.allow_grab_items,
            gui.allow_put_items,
        )));

        player
            .open_handled_screen_direct(screen_handler, gui.title.clone())
            .await;
        Ok(())
    }

    async fn ban(
        &mut self,
        player: Resource<Player>,
        reason: Option<Resource<pumpkin::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        let reason = reason.map(|r| text_component_from_resource(self, &r));
        player.ban(server, reason).await;
        Ok(())
    }

    async fn ban_ip(
        &mut self,
        player: Resource<Player>,
        reason: Option<Resource<pumpkin::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        let reason = reason.map(|r| text_component_from_resource(self, &r));
        player.ban_ip(server, reason).await;
        Ok(())
    }

    async fn get_selected_slot(&mut self, player: Resource<Player>) -> wasmtime::Result<u8> {
        let player = player_from_resource(self, &player)?;
        Ok(player.inventory.get_selected_slot())
    }

    async fn get_item_in_hand(
        &mut self,
        player: Resource<Player>,
        hand: pumpkin::plugin::common::Hand,
    ) -> wasmtime::Result<Option<pumpkin::plugin::common::ItemStack>> {
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
    ) -> wasmtime::Result<Option<pumpkin::plugin::common::ItemStack>> {
        let player = player_from_resource(self, &player)?;
        let slot = slot as usize;
        if slot >= PlayerInventory::MAIN_SIZE {
            return Ok(None);
        }
        let item_stack = player.inventory.main_inventory[slot].lock().await.clone();
        Ok(to_wit_item_stack(&item_stack))
    }

    async fn get_health(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.health.load())
    }

    async fn set_health(&mut self, player: Resource<Player>, health: f32) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_health(health).await;
        Ok(())
    }

    async fn get_max_health(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.get_max_health())
    }

    async fn set_max_health(
        &mut self,
        player: Resource<Player>,
        max_health: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_max_health(max_health).await;
        Ok(())
    }

    async fn get_food_level(&mut self, player: Resource<Player>) -> wasmtime::Result<u8> {
        let player = player_from_resource(self, &player)?;
        Ok(player.hunger_manager.level.load())
    }

    async fn set_food_level(
        &mut self,
        player: Resource<Player>,
        level: u8,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_food_level(level).await;
        Ok(())
    }

    async fn get_saturation(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.hunger_manager.saturation.load())
    }

    async fn set_saturation(
        &mut self,
        player: Resource<Player>,
        saturation: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_saturation(saturation).await;
        Ok(())
    }

    async fn get_exhaustion(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_exhaustion())
    }

    async fn set_exhaustion(
        &mut self,
        player: Resource<Player>,
        exhaustion: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_exhaustion(exhaustion).await;
        Ok(())
    }

    async fn get_absorption(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_absorption())
    }

    async fn set_absorption(
        &mut self,
        player: Resource<Player>,
        absorption: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_absorption(absorption).await;
        Ok(())
    }

    async fn get_experience_level(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_level.load(Ordering::Relaxed))
    }

    async fn get_experience_progress(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_progress.load())
    }

    async fn get_experience_points(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_points.load(Ordering::Relaxed))
    }

    async fn set_experience_level(
        &mut self,
        player: Resource<Player>,
        level: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_experience_level(level, true).await;
        Ok(())
    }

    async fn set_experience_progress(
        &mut self,
        player: Resource<Player>,
        progress: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player
            .set_experience(
                player.experience_level.load(Ordering::Relaxed),
                progress,
                player.experience_points.load(Ordering::Relaxed),
            )
            .await;
        Ok(())
    }

    async fn set_experience_points(
        &mut self,
        player: Resource<Player>,
        points: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player
            .set_experience(
                player.experience_level.load(Ordering::Relaxed),
                player.experience_progress.load(),
                points,
            )
            .await;
        Ok(())
    }

    async fn add_experience_levels(
        &mut self,
        player: Resource<Player>,
        levels: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.add_experience_levels(levels).await;
        Ok(())
    }

    async fn add_experience_points(
        &mut self,
        player: Resource<Player>,
        points: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.add_experience_points(points).await;
        Ok(())
    }

    async fn is_flying(&mut self, player: Resource<Player>) -> wasmtime::Result<bool> {
        let player = player_from_resource(self, &player)?;
        Ok(player.is_flying().await)
    }

    async fn set_flying(&mut self, player: Resource<Player>, flying: bool) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        {
            let mut abilities = player.abilities.lock().await;
            abilities.flying = flying;
        };
        player.send_abilities_update().await;
        Ok(())
    }

    async fn get_abilities(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::player::PlayerAbilities> {
        let player = player_from_resource(self, &player)?;
        let abilities = player.abilities.lock().await;
        Ok(pumpkin::plugin::player::PlayerAbilities {
            invulnerable: abilities.invulnerable,
            flying: abilities.flying,
            allow_flying: abilities.allow_flying,
            creative: abilities.creative,
            allow_modify_world: abilities.allow_modify_world,
            fly_speed: abilities.fly_speed,
            walk_speed: abilities.walk_speed,
        })
    }

    async fn set_abilities(
        &mut self,
        player: Resource<Player>,
        abilities: pumpkin::plugin::player::PlayerAbilities,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        {
            let mut a = player.abilities.lock().await;
            a.invulnerable = abilities.invulnerable;
            a.flying = abilities.flying;
            a.allow_flying = abilities.allow_flying;
            a.creative = abilities.creative;
            a.allow_modify_world = abilities.allow_modify_world;
            a.fly_speed = abilities.fly_speed;
            a.walk_speed = abilities.walk_speed;
        };
        player.send_abilities_update().await;
        Ok(())
    }

    async fn get_ip(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_ip().await)
    }

    async fn get_skin(&mut self, player: Resource<Player>) -> wasmtime::Result<Option<PlayerSkin>> {
        let player = player_from_resource(self, &player)?;
        Ok(player
            .gameprofile
            .properties
            .iter()
            .find(|p| p.name == "textures")
            .map(|p| PlayerSkin {
                value: p.value.clone(),
                signature: p.signature.clone(),
            }))
    }

    async fn get_skin_parts(&mut self, player: Resource<Player>) -> wasmtime::Result<SkinParts> {
        let player = player_from_resource(self, &player)?;
        let mask = player.config.load().skin_parts;
        let mut parts = SkinParts::empty();
        if mask & 0x01 != 0 {
            parts |= SkinParts::CAPE;
        }
        if mask & 0x02 != 0 {
            parts |= SkinParts::JACKET;
        }
        if mask & 0x04 != 0 {
            parts |= SkinParts::LEFT_SLEEVE;
        }
        if mask & 0x08 != 0 {
            parts |= SkinParts::RIGHT_SLEEVE;
        }
        if mask & 0x10 != 0 {
            parts |= SkinParts::LEFT_PANTS_LEG;
        }
        if mask & 0x20 != 0 {
            parts |= SkinParts::RIGHT_PANTS_LEG;
        }
        if mask & 0x40 != 0 {
            parts |= SkinParts::HAT;
        }
        Ok(parts)
    }

    async fn set_skin_parts(
        &mut self,
        player: Resource<Player>,
        parts: SkinParts,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let mut mask = 0u8;
        if parts.contains(SkinParts::CAPE) {
            mask |= 0x01;
        }
        if parts.contains(SkinParts::JACKET) {
            mask |= 0x02;
        }
        if parts.contains(SkinParts::LEFT_SLEEVE) {
            mask |= 0x04;
        }
        if parts.contains(SkinParts::RIGHT_SLEEVE) {
            mask |= 0x08;
        }
        if parts.contains(SkinParts::LEFT_PANTS_LEG) {
            mask |= 0x10;
        }
        if parts.contains(SkinParts::RIGHT_PANTS_LEG) {
            mask |= 0x20;
        }
        if parts.contains(SkinParts::HAT) {
            mask |= 0x40;
        }

        {
            let mut config = (**player.config.load()).clone();
            config.skin_parts = mask;
            player.config.store(Arc::new(config));
        };
        player.send_client_information().await;
        Ok(())
    }

    async fn send_java_packet(
        &mut self,
        player: Resource<Player>,
        packet: pumpkin::plugin::java_packets::ClientboundPacket,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        if let Some(bytes) = crate::plugin::loader::wasm::wasm_host::wit::v0_1::generated_packets::serialize_java_packet(
            &packet, player.client.java().version.load(),
        ) {
            player.client.java().send_packet_now_data(bytes).await;
        }
        Ok(())
    }

    async fn send_bedrock_packet(
        &mut self,
        player: Resource<Player>,
        packet: pumpkin::plugin::bedrock_packets::ClientboundPacket,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        if let Some(bytes) = crate::plugin::loader::wasm::wasm_host::wit::v0_1::generated_packets::serialize_bedrock_packet(
            &packet,
        ) {
            player.client.send_packet_now_data(bytes).await;
        }
        Ok(())
    }

    async fn send_custom_payload(
        &mut self,
        player: Resource<Player>,
        channel: String,
        data: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        if let crate::net::ClientPlatform::Java(_) = player.client {
            player
                .client
                .send_packet_now(&pumpkin_protocol::java::client::play::CCustomPayload::new(
                    &channel, &data,
                ))
                .await;
        }
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
