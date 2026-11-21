use crate::{
    events::{ToFromWasmEvent, cleanup_event, consume_text_component},
    generated_packets,
    pumpkin::plugin::event::{
        BedrockConnectionPacketReceivedEventData, BedrockConnectionPacketSentEventData,
        ClientboundPacket, Event, JavaConnectionPacketReceivedEventData,
        JavaConnectionPacketSentEventData, JavaConnectionState, MapInitializeEventData,
        PacketReceivedEventData, PacketSentEventData, RawPacket, ServerBroadcastEventData,
        ServerCommandEventData, ServerListPingAddress, ServerListPingEventData,
        ServerLoadEventData, ServerLoadType, ServerTickEndEventData, ServerTickStartEventData,
        ServerboundPacket,
    },
};
use pumpkin_core::net::ClientPlatform;
use pumpkin_core::plugin::server::{
    list_ping::ServerListPingEvent,
    map_initialize::MapInitializeEvent,
    packet::{
        BedrockConnectionPacketReceivedEvent, BedrockConnectionPacketSentEvent,
        JavaConnectionPacketReceivedEvent, JavaConnectionPacketSentEvent, PacketReceivedEvent,
        PacketSentEvent,
    },
    server_broadcast::ServerBroadcastEvent,
    server_command::ServerCommandEvent,
    server_load::{LoadType, ServerLoadEvent},
    server_tick_end::ServerTickEndEvent,
    server_tick_start::ServerTickStartEvent,
};
use pumpkin_protocol::ConnectionState as ProtoConnectionState;
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for PacketReceivedEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player_res = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        let packet = match self.player.client.as_ref() {
            // Typed view is 26.3
            // for older clients only valid after the multiversion plugin ran.
            ClientPlatform::Java(_) => generated_packets::deserialize_java_serverbound_packet(
                ProtoConnectionState::Play,
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

        Event::PacketSentEvent(PacketSentEventData {
            player: player_res,
            packet,
            packet_id: self.packet_id,
            raw_payload: self.payload.iter().copied().collect(),
            extra_packets: self
                .extra_packets
                .iter()
                .map(|(id, payload)| RawPacket {
                    packet_id: *id,
                    payload: payload.to_vec(),
                })
                .collect(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::PacketSentEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.extra_packets = data
                .extra_packets
                .into_iter()
                .map(|p| (p.packet_id, p.payload.into()))
                .collect();
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

const fn to_wasm_java_connection_state(state: ProtoConnectionState) -> JavaConnectionState {
    match state {
        ProtoConnectionState::HandShake => JavaConnectionState::Handshake,
        ProtoConnectionState::Status => JavaConnectionState::Status,
        ProtoConnectionState::Login | ProtoConnectionState::Transfer => JavaConnectionState::Login,
        ProtoConnectionState::Config => JavaConnectionState::Config,
        ProtoConnectionState::Play => JavaConnectionState::Play,
    }
}

impl ToFromWasmEvent for JavaConnectionPacketReceivedEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::JavaConnectionPacketReceivedEvent(JavaConnectionPacketReceivedEventData {
            connection_id: self.connection_id,
            version: crate::player::to_wasm_java_version(self.version),
            state: to_wasm_java_connection_state(self.state),
            packet_id: self.packet_id,
            raw_payload: self.payload.to_vec(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::JavaConnectionPacketReceivedEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::JavaConnectionPacketReceivedEvent(_) => {
                panic!(
                    "Modifying packets from WASM is not yet supported in this simple implementation."
                );
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for JavaConnectionPacketSentEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::JavaConnectionPacketSentEvent(JavaConnectionPacketSentEventData {
            connection_id: self.connection_id,
            version: crate::player::to_wasm_java_version(self.version),
            state: to_wasm_java_connection_state(self.state),
            packet_id: self.packet_id,
            raw_payload: self.payload.to_vec(),
            extra_packets: self
                .extra_packets
                .iter()
                .map(|(id, payload)| RawPacket {
                    packet_id: *id,
                    payload: payload.to_vec(),
                })
                .collect(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::JavaConnectionPacketSentEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.extra_packets = data
                .extra_packets
                .into_iter()
                .map(|p| (p.packet_id, p.payload.into()))
                .collect();
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::JavaConnectionPacketSentEvent(_) => {
                panic!(
                    "Modifying packets from WASM is not yet supported in this simple implementation."
                );
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BedrockConnectionPacketReceivedEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BedrockConnectionPacketReceivedEvent(BedrockConnectionPacketReceivedEventData {
            connection_id: self.connection_id,
            protocol_version: self.protocol_version,
            version: crate::player::to_wasm_bedrock_version(self.version),
            packet_id: self.packet_id,
            raw_payload: self.payload.to_vec(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BedrockConnectionPacketReceivedEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BedrockConnectionPacketReceivedEvent(_) => {
                panic!(
                    "Modifying packets from WASM is not yet supported in this simple implementation."
                );
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BedrockConnectionPacketSentEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BedrockConnectionPacketSentEvent(BedrockConnectionPacketSentEventData {
            connection_id: self.connection_id,
            protocol_version: self.protocol_version,
            version: crate::player::to_wasm_bedrock_version(self.version),
            packet_id: self.packet_id,
            raw_payload: self.payload.to_vec(),
            extra_packets: self
                .extra_packets
                .iter()
                .map(|(id, payload)| RawPacket {
                    packet_id: *id,
                    payload: payload.to_vec(),
                })
                .collect(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BedrockConnectionPacketSentEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.extra_packets = data
                .extra_packets
                .into_iter()
                .map(|p| (p.packet_id, p.payload.into()))
                .collect();
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BedrockConnectionPacketSentEvent(_) => {
                panic!(
                    "Modifying packets from WASM is not yet supported in this simple implementation."
                );
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
            .add(self.message.clone())
            .expect("failed to add text-component resource");
        let sender = state
            .add(self.sender.clone())
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
            .add(self.motd.clone())
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
                address:
                    pumpkin_core::plugin::api::events::server::list_ping::ServerListPingAddress::new(
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
    use pumpkin_protocol::packet::MultiVersionJavaPacket;
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
                .get::<TextComponent>(&Resource::new_own(motd_rep))
                .is_err()
        );
    }

    const ACCEPT_TELEPORTATION_26_3: [u8; 33] = [
        0x01, 0x3F, 0xE0, 0, 0, 0, 0, 0, 0, 0x40, 0x52, 0x40, 0, 0, 0, 0, 0, 0x3F, 0xE0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0xC2, 0xB4, 0, 0,
    ];

    #[test]
    fn play_packet_is_not_decoded_as_config_packet_with_the_same_id() {
        let version = pumpkin_data::packet::CURRENT_MC_VERSION;
        let id = pumpkin_protocol::java::server::play::SConfirmTeleport::to_id(version);

        let packet = generated_packets::deserialize_java_serverbound_packet(
            ProtoConnectionState::Play,
            id,
            &ACCEPT_TELEPORTATION_26_3,
            version,
        );

        assert!(packet.is_none());
    }

    #[test]
    fn java_connection_state_mapping_parity() {
        assert_eq!(
            to_wasm_java_connection_state(ProtoConnectionState::HandShake),
            JavaConnectionState::Handshake
        );
        assert_eq!(
            to_wasm_java_connection_state(ProtoConnectionState::Status),
            JavaConnectionState::Status
        );
        assert_eq!(
            to_wasm_java_connection_state(ProtoConnectionState::Login),
            JavaConnectionState::Login
        );
        assert_eq!(
            to_wasm_java_connection_state(ProtoConnectionState::Transfer),
            JavaConnectionState::Login
        );
        assert_eq!(
            to_wasm_java_connection_state(ProtoConnectionState::Config),
            JavaConnectionState::Config
        );
        assert_eq!(
            to_wasm_java_connection_state(ProtoConnectionState::Play),
            JavaConnectionState::Play
        );
    }

    #[test]
    fn java_connection_packet_received_event_marshaling() {
        let mut state = PluginHostState::new();
        let payload = bytes::Bytes::from_static(&[1, 2, 3, 4]);
        let mut event = JavaConnectionPacketReceivedEvent::new(
            42,
            pumpkin_util::version::JavaMinecraftVersion::V_1_20_2,
            ProtoConnectionState::Login,
            0x00,
            payload,
        );

        let wasm_event = event.to_wasm_event(&mut state);
        if let Event::JavaConnectionPacketReceivedEvent(ref data) = wasm_event {
            assert_eq!(data.connection_id, 42);
            assert_eq!(
                data.version,
                crate::pumpkin::plugin::player::JavaMinecraftVersion::V1202
            );
            assert_eq!(data.state, JavaConnectionState::Login);
            assert_eq!(data.packet_id, 0x00);
            assert_eq!(data.raw_payload, vec![1, 2, 3, 4]);
            assert!(!data.cancelled);
        } else {
            panic!("Expected JavaConnectionPacketReceivedEvent");
        }

        let modified_event =
            Event::JavaConnectionPacketReceivedEvent(JavaConnectionPacketReceivedEventData {
                connection_id: 42,
                version: crate::pumpkin::plugin::player::JavaMinecraftVersion::V1202,
                state: JavaConnectionState::Login,
                packet_id: 0x05,
                raw_payload: vec![9, 8, 7],
                cancelled: true,
            });

        event.apply_wasm_event(modified_event, &mut state);
        assert_eq!(event.packet_id, 0x05);
        assert_eq!(&event.payload[..], &[9, 8, 7]);
        assert!(event.cancelled);
    }

    #[test]
    fn java_connection_packet_sent_event_marshaling() {
        let mut state = PluginHostState::new();
        let payload = bytes::Bytes::from_static(&[10, 20]);
        let mut event = JavaConnectionPacketSentEvent::new(
            99,
            pumpkin_util::version::JavaMinecraftVersion::V_1_16_4,
            ProtoConnectionState::HandShake,
            0x01,
            payload,
        );

        let wasm_event = event.to_wasm_event(&mut state);
        if let Event::JavaConnectionPacketSentEvent(ref data) = wasm_event {
            assert_eq!(data.connection_id, 99);
            assert_eq!(
                data.version,
                crate::pumpkin::plugin::player::JavaMinecraftVersion::V1164
            );
            assert_eq!(data.state, JavaConnectionState::Handshake);
            assert_eq!(data.packet_id, 0x01);
            assert_eq!(data.raw_payload, vec![10, 20]);
            assert!(data.extra_packets.is_empty());
            assert!(!data.cancelled);
        } else {
            panic!("Expected JavaConnectionPacketSentEvent");
        }

        let modified_event =
            Event::JavaConnectionPacketSentEvent(JavaConnectionPacketSentEventData {
                connection_id: 99,
                version: crate::pumpkin::plugin::player::JavaMinecraftVersion::V1164,
                state: JavaConnectionState::Handshake,
                packet_id: 0x02,
                raw_payload: vec![30, 40, 50],
                extra_packets: vec![RawPacket {
                    packet_id: 0x03,
                    payload: vec![60],
                }],
                cancelled: true,
            });

        event.apply_wasm_event(modified_event, &mut state);
        assert_eq!(event.packet_id, 0x02);
        assert_eq!(&event.payload[..], &[30, 40, 50]);
        assert_eq!(event.extra_packets.len(), 1);
        assert_eq!(event.extra_packets[0].0, 0x03);
        assert_eq!(&event.extra_packets[0].1[..], &[60]);
        assert!(event.cancelled);
    }

    #[test]
    fn java_version_bijection() {
        use pumpkin_util::version::JavaMinecraftVersion as J;
        let versions = [
            J::V_1_7_2,
            J::V_1_7_6,
            J::V_1_8,
            J::V_1_9,
            J::V_1_9_1,
            J::V_1_9_2,
            J::V_1_9_3,
            J::V_1_10,
            J::V_1_11,
            J::V_1_11_1,
            J::V_1_12,
            J::V_1_12_1,
            J::V_1_12_2,
            J::V_1_13,
            J::V_1_13_1,
            J::V_1_13_2,
            J::V_1_14,
            J::V_1_14_1,
            J::V_1_14_2,
            J::V_1_14_3,
            J::V_1_14_4,
            J::V_1_15,
            J::V_1_15_1,
            J::V_1_15_2,
            J::V_1_16,
            J::V_1_16_1,
            J::V_1_16_2,
            J::V_1_16_3,
            J::V_1_16_4,
            J::V_1_17,
            J::V_1_17_1,
            J::V_1_18,
            J::V_1_18_2,
            J::V_1_19,
            J::V_1_19_1,
            J::V_1_19_3,
            J::V_1_19_4,
            J::V_1_20,
            J::V_1_20_2,
            J::V_1_20_3,
            J::V_1_20_5,
            J::V_1_21,
            J::V_1_21_2,
            J::V_1_21_4,
            J::V_1_21_5,
            J::V_1_21_6,
            J::V_1_21_7,
            J::V_1_21_9,
            J::V_1_21_11,
            J::V_26_1,
            J::V_26_2,
            J::V_26_3,
            J::Unknown,
        ];
        for v in versions {
            let wasm_v = crate::player::to_wasm_java_version(v);
            let round_trip = crate::player::from_wasm_java_version(wasm_v);
            assert_eq!(round_trip, v, "Failed bijection for version {v:?}");
        }
    }

    #[test]
    fn bedrock_connection_packet_received_event_marshaling() {
        let mut state = PluginHostState::new();
        let payload = bytes::Bytes::from_static(&[0xAA, 0xBB]);
        let mut event = BedrockConnectionPacketReceivedEvent::new(
            77,
            671,
            pumpkin_util::version::BedrockMinecraftVersion::V_1_21,
            1,
            payload,
        );

        let wasm_event = event.to_wasm_event(&mut state);
        if let Event::BedrockConnectionPacketReceivedEvent(ref data) = wasm_event {
            assert_eq!(data.connection_id, 77);
            assert_eq!(data.protocol_version, 671);
            assert_eq!(
                data.version,
                crate::pumpkin::plugin::player::BedrockMinecraftVersion::V121
            );
            assert_eq!(data.packet_id, 1);
            assert_eq!(data.raw_payload, vec![0xAA, 0xBB]);
            assert!(!data.cancelled);
        } else {
            panic!("Expected BedrockConnectionPacketReceivedEvent");
        }

        let modified =
            Event::BedrockConnectionPacketReceivedEvent(BedrockConnectionPacketReceivedEventData {
                connection_id: 77,
                protocol_version: 671,
                version: crate::pumpkin::plugin::player::BedrockMinecraftVersion::V121,
                packet_id: 1,
                raw_payload: vec![0xCC, 0xDD, 0xEE],
                cancelled: false,
            });
        event.apply_wasm_event(modified, &mut state);
        assert_eq!(&event.payload[..], &[0xCC, 0xDD, 0xEE]);
        assert!(!event.cancelled);
    }

    #[test]
    fn bedrock_connection_packet_sent_event_marshaling() {
        let mut state = PluginHostState::new();
        let payload = bytes::Bytes::from_static(&[1, 0]);
        let mut event = BedrockConnectionPacketSentEvent::new(
            88,
            2193,
            pumpkin_util::version::BedrockMinecraftVersion::V_1_26_51,
            143,
            payload,
        );

        let wasm_event = event.to_wasm_event(&mut state);
        if let Event::BedrockConnectionPacketSentEvent(ref data) = wasm_event {
            assert_eq!(data.connection_id, 88);
            assert_eq!(data.protocol_version, 2193);
            assert_eq!(
                data.version,
                crate::pumpkin::plugin::player::BedrockMinecraftVersion::V12651
            );
            assert_eq!(data.packet_id, 143);
            assert!(data.extra_packets.is_empty());
        } else {
            panic!("Expected BedrockConnectionPacketSentEvent");
        }

        let modified =
            Event::BedrockConnectionPacketSentEvent(BedrockConnectionPacketSentEventData {
                connection_id: 88,
                protocol_version: 2193,
                version: crate::pumpkin::plugin::player::BedrockMinecraftVersion::V12651,
                packet_id: 143,
                raw_payload: vec![2, 0],
                extra_packets: vec![RawPacket {
                    packet_id: 2,
                    payload: vec![0, 0, 0, 0],
                }],
                cancelled: true,
            });
        event.apply_wasm_event(modified, &mut state);
        assert_eq!(event.packet_id, 143);
        assert_eq!(&event.payload[..], &[2, 0]);
        assert_eq!(event.extra_packets.len(), 1);
        assert_eq!(event.extra_packets[0].0, 2);
        assert_eq!(&event.extra_packets[0].1[..], &[0, 0, 0, 0]);
        assert!(event.cancelled);
    }
}
