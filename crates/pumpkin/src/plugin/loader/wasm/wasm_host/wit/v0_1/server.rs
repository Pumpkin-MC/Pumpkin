use pumpkin_util::text::TextComponent;
use wasmtime::component::Resource;

use crate::command::CommandSender;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::enchantments::{
    CustomEnchantment as WitCustomEnchantment, EnchantmentManager as WitEnchantmentManager,
};
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::recipe::RecipeManager as WitRecipeManager;
use pumpkin::plugin::server::CommandSender as WasmCommandSender;

use super::player::{
    from_wit_permission_level, parse_ban_expiry, text_component_from_resource,
    to_wit_permission_level,
};
use crate::data::SaveJSONConfiguration;
use crate::plugin::{
    loader::wasm::wasm_host::{
        state::{PluginHostState, ServerResource},
        wit::v0_1::pumpkin::{
            self,
            plugin::{
                datapack::DatapackManager as WitDatapackManager,
                player::{BanIpOptions, BanPlayerOptions, Player},
                server::{
                    BanManager as WitBanManager, BannedIpEntry, BannedPlayerEntry, Difficulty,
                    Dimension, OpEntry, OpManager as WitOpManager, Server, SysInfo,
                    WhitelistEntry as WitWhitelistEntry, WhitelistManager as WitWhitelistManager,
                },
                uuid::Uuid as WitUuid,
            },
        },
        wit::v0_1::uuid::UuidExt,
    },
    permissions,
};

impl PluginHostState {
    fn get_server_res(&self, res: &Resource<Server>) -> wasmtime::Result<&ServerResource> {
        self.resource_table
            .get::<ServerResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl pumpkin::plugin::server::Host for PluginHostState {}

impl pumpkin::plugin::server::HostServer for PluginHostState {
    async fn get_sys_info(&mut self, _res: Resource<Server>) -> wasmtime::Result<SysInfo> {
        let has_perm = |p: &str| self.permissions.iter().any(|perm| perm == p);

        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        let cpu_count = (has_perm(permissions::SYS_INFO) || has_perm(permissions::SYS_INFO_CPU))
            .then(|| sys.cpus().len() as u32);

        let (total_memory, used_memory) =
            if has_perm(permissions::SYS_INFO) || has_perm(permissions::SYS_INFO_RAM) {
                (Some(sys.total_memory()), Some(sys.used_memory()))
            } else {
                (None, None)
            };

        let (os_name, os_version) =
            if has_perm(permissions::SYS_INFO) || has_perm(permissions::SYS_INFO_OS) {
                (sysinfo::System::name(), sysinfo::System::os_version())
            } else {
                (None, None)
            };

        Ok(SysInfo {
            cpu_count,
            total_memory,
            used_memory,
            os_name,
            os_version,
            pumpkin_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    async fn get_difficulty(&mut self, res: Resource<Server>) -> wasmtime::Result<Difficulty> {
        let resource = self.get_server_res(&res)?;

        Ok(match resource.provider.get_difficulty() {
            pumpkin_util::Difficulty::Peaceful => Difficulty::Peaceful,
            pumpkin_util::Difficulty::Easy => Difficulty::Easy,
            pumpkin_util::Difficulty::Normal => Difficulty::Normal,
            pumpkin_util::Difficulty::Hard => Difficulty::Hard,
        })
    }

    async fn get_player_count(&mut self, _res: Resource<Server>) -> wasmtime::Result<u32> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.get_player_count() as u32)
    }

    async fn get_mspt(&mut self, _res: Resource<Server>) -> wasmtime::Result<f64> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.get_mspt())
    }

    async fn get_tps(&mut self, _res: Resource<Server>) -> wasmtime::Result<f64> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.get_tps())
    }

    async fn get_all_players(
        &mut self,
        _res: Resource<Server>,
    ) -> wasmtime::Result<Vec<Resource<Player>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .get_all_players()
            .into_iter()
            .map(|player| {
                self.add_player(player)
                    .expect("failed to add player resource")
            })
            .collect())
    }

    async fn get_player_by_name(
        &mut self,
        _rep: Resource<Server>,
        name: String,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server
            .get_player_by_name(&name)
            .map(|player| self.add_player(player))
            .transpose()
    }

    async fn get_player_by_uuid(
        &mut self,
        _rep: Resource<Server>,
        id: WitUuid,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        let uuid = WitUuid::from_wit(&id);

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server
            .get_player_by_uuid(uuid)
            .map(|player| self.add_player(player))
            .transpose()
    }

    async fn get_all_worlds(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Vec<Resource<pumpkin::plugin::world::World>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .worlds
            .load()
            .iter()
            .map(|world| {
                self.add_world(world.clone())
                    .expect("failed to add world resource")
            })
            .collect())
    }

    async fn get_world_by_name(
        &mut self,
        _rep: Resource<Server>,
        name: String,
    ) -> wasmtime::Result<Option<Resource<pumpkin::plugin::world::World>>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .worlds
            .load()
            .iter()
            .find(|world| world.get_world_name() == name || world.dimension.minecraft_name == name)
            .map(|world| {
                self.add_world(world.clone())
                    .expect("failed to add world resource")
            }))
    }

    async fn has_world(&mut self, _rep: Resource<Server>, name: String) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .worlds
            .load()
            .iter()
            .any(|world| world.get_world_name() == name || world.dimension.minecraft_name == name))
    }

    async fn create_world(
        &mut self,
        _rep: Resource<Server>,
        name: String,
        dimension: Dimension,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::world::World>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        let internal_dim = match dimension {
            Dimension::Overworld => pumpkin_data::dimension::Dimension::OVERWORLD,
            Dimension::Nether => pumpkin_data::dimension::Dimension::THE_NETHER,
            Dimension::End => pumpkin_data::dimension::Dimension::THE_END,
        };

        let world = server.create_world(name, internal_dim).await;
        self.add_world(world)
            .map_err(|_| wasmtime::Error::msg("failed to add world resource"))
    }

    async fn unload_world(
        &mut self,
        _rep: Resource<Server>,
        name: String,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.unload_world(&name).await)
    }

    async fn save_all(&mut self, _rep: Resource<Server>) -> wasmtime::Result<Result<(), String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.save_all().await)
    }

    async fn get_players_in_world(
        &mut self,
        _rep: Resource<Server>,
        world: Resource<pumpkin::plugin::world::World>,
    ) -> wasmtime::Result<Vec<Resource<pumpkin::plugin::player::Player>>> {
        let world_res = self.get_world_res(&world)?;
        let players = world_res.provider.players.load();
        let mut player_resources = Vec::with_capacity(players.len());
        for p in players.iter() {
            let res = self.add_player(p.clone())?;
            player_resources.push(res);
        }
        Ok(player_resources)
    }

    async fn get_player_count_in_world(
        &mut self,
        _rep: Resource<Server>,
        world: Resource<pumpkin::plugin::world::World>,
    ) -> wasmtime::Result<u32> {
        let world_res = self.get_world_res(&world)?;
        Ok(world_res.provider.players.load().len() as u32)
    }

    async fn broadcast(&mut self, _rep: Resource<Server>, message: String) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server.broadcast_message(
            &TextComponent::text(message),
            &TextComponent::text("Server"),
            0,
            None,
        );

        Ok(())
    }

    async fn broadcast_tab_list_header_footer(
        &mut self,
        _rep: Resource<Server>,
        header: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        footer: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let header = text_component_from_resource(self, &header);
        let footer = text_component_from_resource(self, &footer);
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server.broadcast_tab_list_header_footer(&header, &footer);
        Ok(())
    }

    async fn execute_command(
        &mut self,
        _rep: Resource<Server>,
        command: String,
        sender: WasmCommandSender,
    ) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        let native_sender = match sender {
            WasmCommandSender::Console => CommandSender::Console,
            WasmCommandSender::Player(player_res) => {
                // Extract the native Player reference from the WASM resource
                let player_resource =
                    self.resource_table
                        .get::<crate::plugin::loader::wasm::wasm_host::state::PlayerResource>(
                        &Resource::new_own(player_res.rep()),
                    )?;

                CommandSender::Player(player_resource.provider.clone())
            }
        };

        let dispatcher = server.command_dispatcher.load();
        dispatcher.handle_command(&native_sender.into_source(server), &command);

        Ok(())
    }

    async fn get_max_players(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u32> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.advanced_config.networking.java.max_players)
    }

    async fn is_hardcore(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.hardcore)
    }

    async fn is_online_mode(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.advanced_config.networking.java.online_mode)
    }

    async fn get_motd(&mut self, _rep: Resource<Server>) -> wasmtime::Result<String> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.advanced_config.networking.java.motd.clone())
    }

    async fn has_whitelist(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.white_list)
    }

    async fn get_allow_nether(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.allow_nether)
    }

    async fn get_allow_end(&mut self, _rep: Resource<Server>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.allow_end)
    }

    async fn get_view_distance(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u8> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.advanced_config.networking.java.view_distance.get())
    }

    async fn get_simulation_distance(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u8> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server
            .advanced_config
            .networking
            .java
            .simulation_distance
            .get())
    }

    async fn get_default_gamemode(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<pumpkin::plugin::common::GameMode> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(super::events::to_wasm_game_mode(
            server.basic_config.default_gamemode,
        ))
    }

    async fn get_recipe_manager(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Resource<WitRecipeManager>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        self.add_recipe_manager(server.recipe_manager.clone())
    }

    async fn get_op_manager(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Resource<WitOpManager>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        self.add_op_manager(server.clone())
    }

    async fn get_ban_manager(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Resource<WitBanManager>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        self.add_ban_manager(server.clone())
    }

    async fn get_whitelist_manager(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Resource<WitWhitelistManager>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        self.add_whitelist_manager(server.clone())
    }

    async fn get_advancement(
        &mut self,
        _rep: Resource<Server>,
        id: String,
    ) -> wasmtime::Result<Option<pumpkin::plugin::advancement::AdvancementInfo>> {
        let Some(advancement) =
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::advancement::find_advancement(&id)
        else {
            return Ok(None);
        };
        crate::plugin::loader::wasm::wasm_host::wit::v0_1::advancement::to_wasm_advancement_info(
            self,
            advancement,
        )
        .map(Some)
    }

    async fn get_all_advancement_ids(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Vec<String>> {
        let ids = pumpkin_data::Advancement::get_identifier_list()
            .iter()
            .map(ToString::to_string)
            .collect();
        Ok(ids)
    }

    async fn get_enchantment_manager(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Resource<WitEnchantmentManager>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        self.add_enchantment_manager(server.enchantment_manager.clone())
    }

    async fn get_enchantment(
        &mut self,
        _rep: Resource<Server>,
        id: String,
    ) -> wasmtime::Result<Option<WitCustomEnchantment>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        if let Some(entry) = server.enchantment_manager.get(&id).await {
            let description = self.add_text_component(entry.description)?;
            return Ok(Some(WitCustomEnchantment {
                id: entry.id,
                description,
                max_level: entry.max_level,
                anvil_cost: entry.anvil_cost,
                supported_items: entry.supported_items,
                weight: entry.weight,
                slots: entry
                    .slots
                    .iter()
                    .map(super::enchantment::to_wit_slot)
                    .collect(),
                exclusive_set: entry.exclusive_set,
            }));
        }

        if let Some(vanilla) = super::enchantment::find_vanilla_enchantment(&id) {
            let description =
                self.add_text_component(TextComponent::translate(vanilla.description, []))?;
            return Ok(Some(WitCustomEnchantment {
                id: vanilla.name.to_string(),
                description,
                max_level: vanilla.max_level.max(1) as u32,
                anvil_cost: vanilla.anvil_cost,
                supported_items: vanilla
                    .supported_items
                    .0
                    .first()
                    .copied()
                    .unwrap_or("")
                    .to_string(),
                weight: vanilla.weight.max(1) as u32,
                slots: vanilla
                    .slots
                    .iter()
                    .map(super::enchantment::to_wit_slot)
                    .collect(),
                exclusive_set: vanilla.exclusive_set.map_or_else(Vec::new, |tag| {
                    tag.0.iter().map(|s| (*s).to_string()).collect()
                }),
            }));
        }

        Ok(None)
    }

    async fn get_all_enchantment_ids(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Vec<String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let mut ids = server.enchantment_manager.get_all_ids().await;
        for enc in pumpkin_data::enchantment::Enchantment::ALL {
            ids.push(enc.name.to_string());
        }
        Ok(ids)
    }

    async fn get_datapack_manager(
        &mut self,
        _rep: Resource<Server>,
    ) -> wasmtime::Result<Resource<WitDatapackManager>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        self.add_datapack_manager(server.clone())
    }

    async fn drop(&mut self, rep: Resource<Server>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ServerResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::server::HostOpManager for PluginHostState {
    async fn is_op(&mut self, _res: Resource<WitOpManager>, id: WitUuid) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let ops = server
            .data
            .operator_config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(ops.get_entry(&uuid).is_some())
    }

    async fn get_op(
        &mut self,
        _res: Resource<WitOpManager>,
        id: WitUuid,
    ) -> wasmtime::Result<Option<OpEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let ops = server
            .data
            .operator_config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(ops.get_entry(&uuid).map(|entry| OpEntry {
            uuid: WitUuid::to_wit(&entry.uuid),
            name: entry.name.clone(),
            level: to_wit_permission_level(entry.level),
            bypasses_player_limit: entry.bypasses_player_limit,
        }))
    }

    async fn get_permission_level(
        &mut self,
        _res: Resource<WitOpManager>,
        id: WitUuid,
    ) -> wasmtime::Result<pumpkin::plugin::permission::PermissionLevel> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let ops = server
            .data
            .operator_config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(ops.get_entry(&uuid).map_or(
            pumpkin::plugin::permission::PermissionLevel::Zero,
            |entry| to_wit_permission_level(entry.level),
        ))
    }

    async fn op_player(
        &mut self,
        _res: Resource<WitOpManager>,
        name: String,
        id: WitUuid,
        level: pumpkin::plugin::permission::PermissionLevel,
        bypasses_player_limit: bool,
    ) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let internal_level = from_wit_permission_level(level);

        {
            let mut config = server
                .data
                .operator_config
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(existing) = config.ops.iter_mut().find(|o| o.uuid == uuid) {
                existing.level = internal_level;
                existing.name.clone_from(&name);
                existing.bypasses_player_limit = bypasses_player_limit;
            } else {
                let op_entry =
                    pumpkin_config::op::Op::new(uuid, name, internal_level, bypasses_player_limit);
                config.ops.push(op_entry);
            }
            config.save();
        };

        if let Some(player) = server.get_player_by_uuid(uuid) {
            let command_dispatcher = server.command_dispatcher.load();
            player.set_permission_lvl(server, internal_level, &command_dispatcher);
        }

        Ok(())
    }

    async fn deop_player(
        &mut self,
        _res: Resource<WitOpManager>,
        id: WitUuid,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);

        let removed = {
            let mut config = server
                .data
                .operator_config
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            config
                .ops
                .iter()
                .position(|o| o.uuid == uuid)
                .is_some_and(|op_index| {
                    config.ops.remove(op_index);
                    config.save();
                    true
                })
        };

        if removed {
            if let Some(player) = server.get_player_by_uuid(uuid) {
                let command_dispatcher = server.command_dispatcher.load();
                player.set_permission_lvl(
                    server,
                    pumpkin_util::PermissionLvl::Zero,
                    &command_dispatcher,
                );
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn list_ops(&mut self, _res: Resource<WitOpManager>) -> wasmtime::Result<Vec<OpEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let config = server
            .data
            .operator_config
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(config
            .ops
            .iter()
            .map(|entry| OpEntry {
                uuid: WitUuid::to_wit(&entry.uuid),
                name: entry.name.clone(),
                level: to_wit_permission_level(entry.level),
                bypasses_player_limit: entry.bypasses_player_limit,
            })
            .collect())
    }

    async fn drop(&mut self, rep: Resource<WitOpManager>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::OpManagerResource>(
                Resource::new_own(rep.rep()),
            );
        Ok(())
    }
}

impl pumpkin::plugin::server::HostBanManager for PluginHostState {
    async fn is_player_banned(
        &mut self,
        _res: Resource<WitBanManager>,
        id: WitUuid,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let now = time::OffsetDateTime::now_utc();
        let mut list = server.data.banned_player_list.write().unwrap();
        list.banned_players
            .retain(|entry| entry.expires.is_none_or(|expires| expires > now));
        list.save();
        Ok(list.banned_players.iter().any(|e| e.uuid == uuid))
    }

    async fn get_player_ban(
        &mut self,
        _res: Resource<WitBanManager>,
        id: WitUuid,
    ) -> wasmtime::Result<Option<BannedPlayerEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let now = time::OffsetDateTime::now_utc();
        let mut list = server.data.banned_player_list.write().unwrap();
        list.banned_players
            .retain(|entry| entry.expires.is_none_or(|expires| expires > now));
        list.save();
        Ok(list
            .banned_players
            .iter()
            .find(|e| e.uuid == uuid)
            .map(|e| BannedPlayerEntry {
                uuid: WitUuid::to_wit(&e.uuid),
                name: e.name.clone(),
                created: e
                    .created
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                source: e.source.clone(),
                expires: e.expires.and_then(|exp| {
                    exp.format(&time::format_description::well_known::Rfc3339)
                        .ok()
                }),
                reason: e.reason.clone(),
            }))
    }

    async fn ban_player(
        &mut self,
        _res: Resource<WitBanManager>,
        name: String,
        id: WitUuid,
        options: BanPlayerOptions,
    ) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let reason_text = options
            .reason
            .as_ref()
            .map(|res| text_component_from_resource(self, res))
            .map_or_else(
                || "Banned by plugin.".to_string(),
                pumpkin_util::text::TextComponent::to_pretty_console,
            );
        let source_name = options.source.unwrap_or_else(|| "Plugin".to_string());
        let expires = parse_ban_expiry(options.expires_at_utc, options.duration_seconds);

        {
            let mut list = server.data.banned_player_list.write().unwrap();
            if let Some(existing) = list.banned_players.iter_mut().find(|e| e.uuid == uuid) {
                existing.name.clone_from(&name);
                existing.source = source_name;
                existing.expires = expires;
                existing.reason.clone_from(&reason_text);
            } else {
                let entry = crate::data::banlist_serializer::BannedPlayerEntry {
                    uuid,
                    name: name.clone(),
                    created: time::OffsetDateTime::now_utc(),
                    source: source_name,
                    expires,
                    reason: reason_text.clone(),
                };
                list.banned_players.push(entry);
            }
            list.save();
        };

        if options.kick_if_online
            && let Some(player) = server.get_player_by_uuid(uuid)
        {
            player.kick(
                crate::net::DisconnectReason::Kicked,
                &pumpkin_util::text::TextComponent::text(reason_text.clone()),
            );
        }

        if options.log_to_console {
            tracing::info!("Banned player {} ({}): {}", name, uuid, reason_text);
        }
        Ok(())
    }

    async fn unban_player(
        &mut self,
        _res: Resource<WitBanManager>,
        id: WitUuid,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let mut list = server.data.banned_player_list.write().unwrap();
        Ok(list
            .banned_players
            .iter()
            .position(|e| e.uuid == uuid)
            .is_some_and(|pos| {
                list.banned_players.remove(pos);
                list.save();
                true
            }))
    }

    async fn list_player_bans(
        &mut self,
        _res: Resource<WitBanManager>,
    ) -> wasmtime::Result<Vec<BannedPlayerEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let now = time::OffsetDateTime::now_utc();
        let mut list = server.data.banned_player_list.write().unwrap();
        list.banned_players
            .retain(|entry| entry.expires.is_none_or(|expires| expires > now));
        list.save();
        Ok(list
            .banned_players
            .iter()
            .map(|e| BannedPlayerEntry {
                uuid: WitUuid::to_wit(&e.uuid),
                name: e.name.clone(),
                created: e
                    .created
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                source: e.source.clone(),
                expires: e.expires.and_then(|exp| {
                    exp.format(&time::format_description::well_known::Rfc3339)
                        .ok()
                }),
                reason: e.reason.clone(),
            })
            .collect())
    }

    async fn is_ip_banned(
        &mut self,
        _res: Resource<WitBanManager>,
        ip: String,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let ip_addr: std::net::IpAddr = ip
            .parse()
            .map_err(|_| wasmtime::Error::msg("Invalid IP address"))?;
        let now = time::OffsetDateTime::now_utc();
        let mut list = server.data.banned_ip_list.write().unwrap();
        list.banned_ips
            .retain(|entry| entry.expires.is_none_or(|expires| expires > now));
        list.save();
        Ok(list.banned_ips.iter().any(|e| e.ip == ip_addr))
    }

    async fn get_ip_ban(
        &mut self,
        _res: Resource<WitBanManager>,
        ip: String,
    ) -> wasmtime::Result<Option<BannedIpEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let ip_addr: std::net::IpAddr = ip
            .parse()
            .map_err(|_| wasmtime::Error::msg("Invalid IP address"))?;
        let now = time::OffsetDateTime::now_utc();
        let mut list = server.data.banned_ip_list.write().unwrap();
        list.banned_ips
            .retain(|entry| entry.expires.is_none_or(|expires| expires > now));
        list.save();
        Ok(list
            .banned_ips
            .iter()
            .find(|e| e.ip == ip_addr)
            .map(|e| BannedIpEntry {
                ip: e.ip.to_string(),
                created: e
                    .created
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                source: e.source.clone(),
                expires: e.expires.and_then(|exp| {
                    exp.format(&time::format_description::well_known::Rfc3339)
                        .ok()
                }),
                reason: e.reason.clone(),
            }))
    }

    async fn ban_ip(
        &mut self,
        _res: Resource<WitBanManager>,
        ip: String,
        options: BanIpOptions,
    ) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let ip_addr: std::net::IpAddr = ip
            .parse()
            .map_err(|_| wasmtime::Error::msg("Invalid IP address"))?;
        let reason_text = options
            .reason
            .as_ref()
            .map(|res| text_component_from_resource(self, res))
            .map_or_else(
                || "Banned by plugin.".to_string(),
                pumpkin_util::text::TextComponent::to_pretty_console,
            );
        let source_name = options.source.unwrap_or_else(|| "Plugin".to_string());
        let expires = parse_ban_expiry(options.expires_at_utc, options.duration_seconds);

        {
            let mut list = server.data.banned_ip_list.write().unwrap();
            if let Some(existing) = list.banned_ips.iter_mut().find(|e| e.ip == ip_addr) {
                existing.source = source_name;
                existing.expires = expires;
                existing.reason.clone_from(&reason_text);
            } else {
                let entry = crate::data::banlist_serializer::BannedIpEntry {
                    ip: ip_addr,
                    created: time::OffsetDateTime::now_utc(),
                    source: source_name,
                    expires,
                    reason: reason_text.clone(),
                };
                list.banned_ips.push(entry);
            }
            list.save();
        };

        if options.kick_matching_players {
            for player in server.get_all_players() {
                if player.client.address().ip() == ip_addr {
                    player.kick(
                        crate::net::DisconnectReason::Kicked,
                        &pumpkin_util::text::TextComponent::text(reason_text.clone()),
                    );
                }
            }
        }

        if options.log_to_console {
            tracing::info!("Banned IP {}: {}", ip_addr, reason_text);
        }
        Ok(())
    }

    async fn unban_ip(
        &mut self,
        _res: Resource<WitBanManager>,
        ip: String,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let ip_addr: std::net::IpAddr = ip
            .parse()
            .map_err(|_| wasmtime::Error::msg("Invalid IP address"))?;
        let mut list = server.data.banned_ip_list.write().unwrap();
        Ok(list
            .banned_ips
            .iter()
            .position(|e| e.ip == ip_addr)
            .is_some_and(|pos| {
                list.banned_ips.remove(pos);
                list.save();
                true
            }))
    }

    async fn list_ip_bans(
        &mut self,
        _res: Resource<WitBanManager>,
    ) -> wasmtime::Result<Vec<BannedIpEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let now = time::OffsetDateTime::now_utc();
        let mut list = server.data.banned_ip_list.write().unwrap();
        list.banned_ips
            .retain(|entry| entry.expires.is_none_or(|expires| expires > now));
        list.save();
        Ok(list
            .banned_ips
            .iter()
            .map(|e| BannedIpEntry {
                ip: e.ip.to_string(),
                created: e
                    .created
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                source: e.source.clone(),
                expires: e.expires.and_then(|exp| {
                    exp.format(&time::format_description::well_known::Rfc3339)
                        .ok()
                }),
                reason: e.reason.clone(),
            })
            .collect())
    }

    async fn drop(&mut self, rep: Resource<WitBanManager>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::BanManagerResource>(
            Resource::new_own(rep.rep()),
        );
        Ok(())
    }
}

impl pumpkin::plugin::server::HostWhitelistManager for PluginHostState {
    async fn is_enabled(&mut self, _res: Resource<WitWhitelistManager>) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.white_list.load(std::sync::atomic::Ordering::Relaxed))
    }

    async fn set_enabled(
        &mut self,
        _res: Resource<WitWhitelistManager>,
        enabled: bool,
    ) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        server
            .white_list
            .store(enabled, std::sync::atomic::Ordering::Relaxed);
        if enabled && server.basic_config.enforce_whitelist {
            let to_kick: Vec<_> = {
                let whitelist = server.data.whitelist_config.read().unwrap();
                server
                    .get_all_players()
                    .into_iter()
                    .filter(|player| !whitelist.is_whitelisted(&player.gameprofile))
                    .collect()
            };
            for player in to_kick {
                player.kick(
                    crate::net::DisconnectReason::Kicked,
                    &pumpkin_macros::translate_cross!(
                        pumpkin_data::translation::java::MULTIPLAYER_DISCONNECT_NOT_WHITELISTED,
                        pumpkin_data::translation::bedrock::DISCONNECT_KICKED
                    ),
                );
            }
        }
        Ok(())
    }

    async fn is_whitelisted(
        &mut self,
        _res: Resource<WitWhitelistManager>,
        id: WitUuid,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let whitelist = server.data.whitelist_config.read().unwrap();
        Ok(whitelist.whitelist.iter().any(|e| e.uuid == uuid))
    }

    async fn add_player(
        &mut self,
        _res: Resource<WitWhitelistManager>,
        name: String,
        id: WitUuid,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let mut config = server.data.whitelist_config.write().unwrap();
        if config.whitelist.iter().any(|e| e.uuid == uuid) {
            Ok(false)
        } else {
            config
                .whitelist
                .push(pumpkin_config::whitelist::WhitelistEntry::new(uuid, name));
            config.save();
            Ok(true)
        }
    }

    async fn remove_player(
        &mut self,
        _res: Resource<WitWhitelistManager>,
        id: WitUuid,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let uuid = WitUuid::from_wit(&id);
        let mut config = server.data.whitelist_config.write().unwrap();
        Ok(config
            .whitelist
            .iter()
            .position(|e| e.uuid == uuid)
            .is_some_and(|pos| {
                config.whitelist.remove(pos);
                config.save();
                true
            }))
    }

    async fn list_entries(
        &mut self,
        _res: Resource<WitWhitelistManager>,
    ) -> wasmtime::Result<Vec<WitWhitelistEntry>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let config = server.data.whitelist_config.read().unwrap();
        Ok(config
            .whitelist
            .iter()
            .map(|e| WitWhitelistEntry {
                uuid: WitUuid::to_wit(&e.uuid),
                name: e.name.clone(),
            })
            .collect())
    }

    async fn drop(&mut self, rep: Resource<WitWhitelistManager>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::WhitelistManagerResource>(
            Resource::new_own(rep.rep()),
        );
        Ok(())
    }
}
