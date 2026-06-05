use crate::net::ClientPlatform;
use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, consume_text_component},
            generated_packets,
            pumpkin::plugin::event::{
                ClientboundPacket, Event, PacketReceivedEventData, PacketSentEventData,
                ServerBroadcastEventData, ServerCommandEventData, ServerTickEndEventData,
                ServerTickStartEventData, ServerboundPacket,
            },
        },
    },
    server::{
        packet::{PacketReceivedEvent, PacketSentEvent},
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
    },
};

impl ToFromWasmEvent for PacketReceivedEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_res = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        let packet = match &self.player.client {
            ClientPlatform::Java(client) => {
                let version = client.version.load();
                let wit_packet = generated_packets::deserialize_java_serverbound_packet(
                    self.packet_id,
                    &self.payload,
                    version,
                );
                wit_packet.map(ServerboundPacket::Java)
            }
            ClientPlatform::Bedrock(_) => {
                let wit_packet = generated_packets::deserialize_bedrock_serverbound_packet(
                    self.packet_id,
                    &self.payload,
                );
                wit_packet.map(ServerboundPacket::Bedrock)
            }
        };

        let packet = packet.expect("Failed to deserialize serverbound packet to WIT. Ensure the packet is supported in the WIT API.");

        Event::PacketReceivedEvent(PacketReceivedEventData {
            player: player_res,
            packet,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::PacketReceivedEvent(_) => {
                // TODO: Implement converting from WIT variant back to raw if needed.
                // For now, we only support cancellation.
                panic!(
                    "Modifying packets from WASM is not yet supported in this simple implementation."
                );
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PacketSentEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_res = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        let packet = match &self.player.client {
            ClientPlatform::Java(_) => {
                let wit_packet =
                    generated_packets::clientbound_java_any_to_wit(self.packet.as_ref());
                wit_packet.map(ClientboundPacket::Java)
            }
            ClientPlatform::Bedrock(_) => {
                let wit_packet =
                    generated_packets::clientbound_bedrock_any_to_wit(self.packet.as_ref());
                wit_packet.map(ClientboundPacket::Bedrock)
            }
        };

        let packet = packet.expect("Failed to convert clientbound packet to WIT. Ensure the packet is supported in the WIT API and ToWit is generated.");

        Event::PacketSentEvent(PacketSentEventData {
            player: player_res,
            packet,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::PacketSentEvent(_) => {
                panic!("Modifying packets from WASM is not yet supported.");
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
