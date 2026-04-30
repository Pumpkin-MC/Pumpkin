use std::sync::Arc;

use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::text::TextComponent;
use uuid::Uuid;
use wasmtime::component::Resource;

use crate::command::CommandSender;
use pumpkin::plugin::server::CommandSender as WasmCommandSender;

use super::player::text_component_from_resource;
use crate::plugin::{
    loader::wasm::wasm_host::{
        state::{PluginHostState, ServerResource},
        wit::v0_1::pumpkin::{
            self,
            plugin::{
                player::Player,
                server::{Difficulty, Dimension, Server, SysInfo},
            },
        },
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
        id: String,
    ) -> wasmtime::Result<Option<Resource<Player>>> {
        let Ok(uuid) = Uuid::parse_str(&id) else {
            return Ok(None);
        };

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
            .find(|world| world.dimension.minecraft_name == name)
            .map(|world| {
                self.add_world(world.clone())
                    .expect("failed to add world resource")
            }))
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

    async fn broadcast(&mut self, _rep: Resource<Server>, message: String) -> wasmtime::Result<()> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        server
            .broadcast_message(
                &TextComponent::text(message),
                &TextComponent::text("Server"),
                0,
                None,
            )
            .await;

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
        server
            .broadcast_tab_list_header_footer(&header, &footer)
            .await;
        Ok(())
    }

    async fn broadcast_packet(
        &mut self,
        res: Resource<Server>,
        packet: pumpkin::plugin::packet::Packet,
    ) -> wasmtime::Result<()> {
        let resource = self.get_server_res(&res)?;
        if self.should_defer_effects() {
            self.defer_effect(
                crate::plugin::loader::wasm::wasm_host::state::PendingEffect::ServerBroadcastCustomPayload {
                    server: Arc::clone(&resource.provider),
                    channel: packet.channel,
                    data: packet.data,
                },
            );
            return Ok(());
        }
        for player in resource.provider.get_all_players() {
            player
                .send_custom_payload(&packet.channel, &packet.data)
                .await;
        }
        Ok(())
    }

    async fn broadcast_raw_packet(
        &mut self,
        res: Resource<Server>,
        packet: pumpkin::plugin::packet::RawPacket,
    ) -> wasmtime::Result<()> {
        let resource = self.get_server_res(&res)?;
        if self.should_defer_effects() {
            self.defer_effect(
                crate::plugin::loader::wasm::wasm_host::state::PendingEffect::ServerBroadcastRawPacket {
                    server: Arc::clone(&resource.provider),
                    id: packet.id,
                    payload: packet.payload,
                },
            );
            return Ok(());
        }
        for player in resource.provider.get_all_players() {
            match &player.client {
                crate::net::ClientPlatform::Java(java) => {
                    let mut buf = Vec::new();
                    VarInt(packet.id)
                        .encode(&mut buf)
                        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                    buf.extend_from_slice(&packet.payload);
                    java.enqueue_packet_data(buf.clone().into()).await;
                }
                crate::net::ClientPlatform::Bedrock(bedrock) => {
                    bedrock
                        .send_raw_game_packet(packet.id, packet.payload.clone())
                        .await
                        .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                }
            }
        }
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

        let dispatcher = server.command_dispatcher.read().await;
        dispatcher
            .handle_command(&native_sender.into_source(server).await, &command)
            .await;

        Ok(())
    }

    async fn get_max_players(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u32> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.max_players)
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

        Ok(server.advanced_config.networking.authentication.enabled)
    }

    async fn get_motd(&mut self, _rep: Resource<Server>) -> wasmtime::Result<String> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.motd.clone())
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

        Ok(server.basic_config.view_distance.get())
    }

    async fn get_simulation_distance(&mut self, _rep: Resource<Server>) -> wasmtime::Result<u8> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        Ok(server.basic_config.simulation_distance.get())
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

    async fn drop(&mut self, rep: Resource<Server>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ServerResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
