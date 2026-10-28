use crate::{
    events::{ToFromWasmEvent, cleanup_event, consume_text_component},
    generated_packets,
    pumpkin::plugin::event::{
        ClientboundPacket, Event, MapInitializeEvent as MapInitializeEventData,
        PacketReceivedEvent as PacketReceivedEventData, PacketSentEvent as PacketSentEventData,
        ServerBroadcastEvent as ServerBroadcastEventData,
        ServerCommandEvent as ServerCommandEventData, ServerListPingAddress,
        ServerListPingEvent as ServerListPingEventData, ServerLoadEvent as ServerLoadEventData,
        ServerLoadType, ServerTickEndEvent as ServerTickEndEventData,
        ServerTickStartEvent as ServerTickStartEventData, ServerboundPacket,
    },
};
use pumpkin_core::net::ClientPlatform;
use pumpkin_core::plugin::server::{
    list_ping::ServerListPingEvent,
    map_initialize::MapInitializeEvent,
    packet::{PacketReceivedEvent, PacketSentEvent},
    server_broadcast::ServerBroadcastEvent,
    server_command::ServerCommandEvent,
    server_load::{LoadType, ServerLoadEvent},
    server_tick_end::ServerTickEndEvent,
    server_tick_start::ServerTickStartEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for PacketReceivedEvent {
    type Wit = PacketReceivedEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player_res = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        let packet = match self.player.client.as_ref() {
            ClientPlatform::Java(_) => generated_packets::deserialize_java_serverbound_packet(
                self.packet_id,
                &self.payload,
                pumpkin_data::packet::CURRENT_MC_VERSION,
            )
            .map_or(ServerboundPacket::Unknown, ServerboundPacket::Java),
            ClientPlatform::Bedrock(_) => {
                generated_packets::deserialize_bedrock_serverbound_packet(
                    self.packet_id,
                    &self.payload,
                )
                .map_or(ServerboundPacket::Unknown, ServerboundPacket::Bedrock)
            }
        };

        Self::Wit {
            player: player_res,
            packet,
            packet_id: self.packet_id,
            raw_payload: self.payload.to_vec(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.packet_id = data.packet_id;
        self.payload = data.raw_payload.into();
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Modifying packets from WASM is not yet supported in this simple implementation.");
    }
}

impl ToFromWasmEvent for PacketSentEvent {
    type Wit = PacketSentEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player_res = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        let packet = match self.player.client.as_ref() {
            ClientPlatform::Java(_) => {
                generated_packets::clientbound_java_any_to_wit(self.packet.as_ref())
                    .map_or(ClientboundPacket::Unknown, ClientboundPacket::Java)
            }
            ClientPlatform::Bedrock(_) => {
                generated_packets::clientbound_bedrock_any_to_wit(self.packet.as_ref())
                    .map_or(ClientboundPacket::Unknown, ClientboundPacket::Bedrock)
            }
        };

        Self::Wit {
            player: player_res,
            packet,
            packet_id: self.packet_id,
            raw_payload: self.payload.iter().copied().collect(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.packet_id = data.packet_id;
        self.payload = data.raw_payload.into();
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Modifying packets from WASM is not yet supported.");
    }
}

impl ToFromWasmEvent for ServerCommandEvent {
    type Wit = ServerCommandEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            command: self.command.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.command = data.command;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            command: data.command,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ServerBroadcastEvent {
    type Wit = ServerBroadcastEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let message = state
            .add(self.message.clone())
            .expect("failed to add text-component resource");
        let sender = state
            .add(self.sender.clone())
            .expect("failed to add text-component resource");

        Self::Wit {
            message,
            sender,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.message = consume_text_component(state, &data.message);
        self.sender = consume_text_component(state, &data.sender);
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            message: consume_text_component(state, &data.message),
            sender: consume_text_component(state, &data.sender),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ServerListPingEvent {
    type Wit = ServerListPingEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let motd = state
            .add(self.motd.clone())
            .expect("failed to add text-component resource");

        Self::Wit {
            hostname: self.hostname().to_string(),
            address: ServerListPingAddress {
                host: self.address().host().to_string(),
                port: self.address().port(),
            },
            motd,
            max_players: self.max_players,
            num_players: self.num_players,
            favicon: self.favicon.clone(),
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        let returned = Self::from_wasm_event(data, state);
        self.motd = returned.motd;
        self.max_players = returned.max_players;
        self.num_players = returned.num_players;
        self.favicon = returned.favicon;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            hostname: data.hostname,
            address:
                pumpkin_core::plugin::api::events::server::list_ping::ServerListPingAddress::new(
                    data.address.host,
                    data.address.port,
                ),
            motd: consume_text_component(state, &data.motd),
            max_players: data.max_players,
            num_players: data.num_players,
            favicon: data.favicon,
        }
    }
}

impl ToFromWasmEvent for ServerLoadEvent {
    type Wit = ServerLoadEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            load_type: match self.load_type {
                LoadType::Startup => ServerLoadType::Startup,
                LoadType::Reload => ServerLoadType::Reload,
            },
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            load_type: match data.load_type {
                ServerLoadType::Startup => LoadType::Startup,
                ServerLoadType::Reload => LoadType::Reload,
            },
        }
    }
}

impl ToFromWasmEvent for ServerTickEndEvent {
    type Wit = ServerTickEndEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            tick: self.tick,
            duration_nanos: self.duration_nanos,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            tick: data.tick,
            duration_nanos: data.duration_nanos,
        }
    }
}

impl ToFromWasmEvent for ServerTickStartEvent {
    type Wit = ServerTickStartEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit { tick: self.tick }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self { tick: data.tick }
    }
}

impl ToFromWasmEvent for MapInitializeEvent {
    type Wit = MapInitializeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            map_id: self.map_id,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            map_id: data.map_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::text::TextComponent;
    use wasmtime::component::Resource;

    #[test]
    fn server_list_ping_applies_and_consumes_returned_resources() {
        let mut state = PluginHostState::new();
        let original_motd = TextComponent::text("Original");
        let returned_motd = TextComponent::text("Returned");
        let mut event = ServerListPingEvent::new(
            "original.example".to_string(),
            "127.0.0.1:25565"
                .parse()
                .expect("test address should parse"),
            original_motd,
            20,
            1,
            None,
        );
        let motd = state
            .add(returned_motd.clone())
            .expect("text component resource should be inserted");
        let motd_rep = motd.rep();
        let returned = ServerListPingEventData {
            hostname: "replacement.example".to_string(),
            address: ServerListPingAddress {
                host: "192.0.2.1".to_string(),
                port: 25_566,
            },
            motd,
            max_players: 40,
            num_players: 2,
            favicon: Some("data:image/png;base64,test".to_string()),
        };

        event.apply_wasm_event(returned, &mut state);

        assert_eq!(event.hostname(), "original.example");
        assert_eq!(event.address().host(), "127.0.0.1");
        assert_eq!(event.address().port(), 25_565);
        assert_eq!(event.motd, returned_motd);
        assert_eq!(event.max_players, 40);
        assert_eq!(event.num_players, 2);
        assert_eq!(event.favicon.as_deref(), Some("data:image/png;base64,test"));
        assert!(
            state
                .resource_table
                .get::<TextComponent>(&Resource::new_own(motd_rep))
                .is_err()
        );
    }
}
