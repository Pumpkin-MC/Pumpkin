use crate::net::ClientPlatform;
use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, consume_player, consume_text_component},
            generated_packets,
            pumpkin::plugin::event::{
                ClientboundPacket, Event, PacketReceivedEventData, PacketSentEventData,
                ServerBroadcastEventData, ServerCommandEventData, ServerLoadEventData,
                ServerLoadType, ServerTickEndEventData, ServerTickStartEventData,
                ServerboundPacket,
            },
        },
    },
    server::{
        packet::{PacketReceivedEvent, PacketSentEvent},
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_load::{LoadType, ServerLoadEvent},
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
    },
};
use bytes::Bytes;

impl ToFromWasmEvent for PacketReceivedEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_res = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        let packet = match self.player.client.as_ref() {
            ClientPlatform::Java(client) => {
                let version = client.version.load();
                generated_packets::deserialize_java_serverbound_packet(
                    self.packet_id,
                    &self.payload,
                    version,
                )
                .map_or(ServerboundPacket::Unknown, ServerboundPacket::Java)
            }
            ClientPlatform::Bedrock(_) => {
                generated_packets::deserialize_bedrock_serverbound_packet(
                    self.packet_id,
                    &self.payload,
                )
                .map_or(ServerboundPacket::Unknown, ServerboundPacket::Bedrock)
            }
        };

        Event::PacketReceivedEvent(PacketReceivedEventData {
            player: player_res,
            packet,
            packet_id: self.packet_id,
            raw_payload: self.payload.to_vec(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PacketReceivedEvent(data) => Self {
                player: consume_player(state, &data.player),
                packet_id: data.packet_id,
                payload: Bytes::from(data.raw_payload),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PacketSentEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_res = state
            .add_player(self.player.clone())
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

        Event::PacketSentEvent(PacketSentEventData {
            player: player_res,
            packet,
            packet_id: self.packet_id,
            raw_payload: self.payload.iter().copied().collect(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PacketSentEvent(data) => {
                struct RawPacket;

                Self {
                    player: consume_player(state, &data.player),
                    packet_id: data.packet_id,
                    payload: Bytes::from(data.raw_payload),
                    packet: std::sync::Arc::new(RawPacket),
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ServerCommandEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ServerCommandEvent(ServerCommandEventData {
            command: self.command.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerCommandEvent(data) => Self {
                command: data.command,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ServerBroadcastEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let message = state
            .add_text_component(self.message.clone())
            .expect("failed to add text-component resource");
        let sender = state
            .add_text_component(self.sender.clone())
            .expect("failed to add text-component resource");

        Event::ServerBroadcastEvent(ServerBroadcastEventData {
            message,
            sender,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerBroadcastEvent(data) => Self {
                message: consume_text_component(state, &data.message),
                sender: consume_text_component(state, &data.sender),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ServerLoadEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ServerLoadEvent(ServerLoadEventData {
            load_type: match self.load_type {
                LoadType::Startup => ServerLoadType::Startup,
                LoadType::Reload => ServerLoadType::Reload,
            },
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerLoadEvent(data) => Self {
                load_type: match data.load_type {
                    ServerLoadType::Startup => LoadType::Startup,
                    ServerLoadType::Reload => LoadType::Reload,
                },
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ServerTickEndEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ServerTickEndEvent(ServerTickEndEventData {
            tick: self.tick,
            duration_nanos: self.duration_nanos,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerTickEndEvent(data) => Self {
                tick: data.tick,
                duration_nanos: data.duration_nanos,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ServerTickStartEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ServerTickStartEvent(ServerTickStartEventData { tick: self.tick })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerTickStartEvent(data) => Self { tick: data.tick },
            _ => panic!("unexpected event type"),
        }
    }
}
