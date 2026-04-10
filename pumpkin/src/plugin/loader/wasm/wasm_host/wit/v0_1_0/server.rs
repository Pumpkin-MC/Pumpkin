use std::sync::Arc;

use uuid::Uuid;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PluginHostState, ServerResource},
    wit::v0_1_0::pumpkin::{
        self,
        plugin::{
            packet::RawPacket as WasmRawPacket,
            player::Player,
            server::{Difficulty, Server},
        },
    },
};
use pumpkin_protocol::codec::var_int::VarInt;

impl PluginHostState {
    fn get_server_res(&self, res: &Resource<Server>) -> wasmtime::Result<&ServerResource> {
        self.resource_table
            .get::<ServerResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl pumpkin::plugin::server::Host for PluginHostState {}

impl pumpkin::plugin::server::HostServer for PluginHostState {
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

    async fn broadcast_packet(
        &mut self,
        res: Resource<Server>,
        packet: pumpkin::plugin::packet::Packet,
    ) -> wasmtime::Result<()> {
        let resource = self.get_server_res(&res)?;
        if self.should_defer_effects() {
            self.defer_effect(crate::plugin::loader::wasm::wasm_host::state::PendingEffect::ServerBroadcastCustomPayload {
                server: Arc::clone(&resource.provider),
                channel: packet.channel,
                data: packet.data,
            });
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
        packet: WasmRawPacket,
    ) -> wasmtime::Result<()> {
        let resource = self.get_server_res(&res)?;
        if self.should_defer_effects() {
            self.defer_effect(crate::plugin::loader::wasm::wasm_host::state::PendingEffect::ServerBroadcastRawPacket {
                server: Arc::clone(&resource.provider),
                id: packet.id,
                payload: packet.payload,
            });
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

    async fn drop(&mut self, rep: Resource<Server>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ServerResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
