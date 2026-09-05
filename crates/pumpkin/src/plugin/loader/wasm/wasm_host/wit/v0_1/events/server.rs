use crate::net::ClientPlatform;
use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, cleanup_event, consume_text_component},
            generated_packets,
            pumpkin::plugin::event::{
                ClientboundPacket, Event, MapInitializeEventData, PacketReceivedEventData,
                PacketSentEventData, ServerBroadcastEventData, ServerCommandEventData,
                ServerListPingAddress, ServerListPingEventData, ServerLoadEventData,
                ServerLoadType, ServerTickEndEventData, ServerTickStartEventData,
                ServerboundPacket,
            },
        },
    },
    server::{
        list_ping::ServerListPingEvent,
        map_initialize::MapInitializeEvent,
        packet::{PacketReceivedEvent, PacketSentEvent},
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_load::{LoadType, ServerLoadEvent},
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
    },
};

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

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::PacketReceivedEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.cancelled = data.cancelled;
        }
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

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::PacketSentEvent(data) = event {
            self.payload = data.raw_payload.into();
            self.cancelled = data.cancelled;
        }
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

impl ToFromWasmEvent for ServerListPingEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let motd = state
            .add_text_component(self.motd.clone())
            .expect("failed to add text-component resource");

        Event::ServerListPingEvent(ServerListPingEventData {
            hostname: self.hostname().to_string(),
            address: ServerListPingAddress {
                host: self.address().host().to_string(),
                port: self.address().port(),
            },
            motd,
            max_players: self.max_players,
            num_players: self.num_players,
            favicon: self.favicon.clone(),
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerListPingEvent(data) => Self {
                hostname: data.hostname,
                address: crate::plugin::api::events::server::list_ping::ServerListPingAddress::new(
                    data.address.host,
                    data.address.port,
                ),
                motd: consume_text_component(state, &data.motd),
                max_players: data.max_players,
                num_players: data.num_players,
                favicon: data.favicon,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        if !matches!(&event, Event::ServerListPingEvent(_)) {
            cleanup_event(&event, state);
            panic!("unexpected event type");
        }

        let returned = Self::from_wasm_event(event, state);
        self.motd = returned.motd;
        self.max_players = returned.max_players;
        self.num_players = returned.num_players;
        self.favicon = returned.favicon;
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

impl ToFromWasmEvent for MapInitializeEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::MapInitializeEvent(MapInitializeEventData {
            map_id: self.map_id,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::MapInitializeEvent(data) => Self {
                map_id: data.map_id,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::loader::wasm::wasm_host::state::TextComponentResource;
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
            .add_text_component(returned_motd.clone())
            .expect("text component resource should be inserted");
        let motd_rep = motd.rep();
        let returned = Event::ServerListPingEvent(ServerListPingEventData {
            hostname: "replacement.example".to_string(),
            address: ServerListPingAddress {
                host: "192.0.2.1".to_string(),
                port: 25_566,
            },
            motd,
            max_players: 40,
            num_players: 2,
            favicon: Some("data:image/png;base64,test".to_string()),
        });

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
                .get::<TextComponentResource>(&Resource::new_own(motd_rep))
                .is_err()
        );
    }
}
