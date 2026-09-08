use super::super::user::to_wasm_state;
use crate::net::user::Edition;
use crate::plugin::server::packet::decode_packet;
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
        let user = state
            .add_user(self.user.clone())
            .expect("failed to add user resource");

        let packet = match self.user.edition {
            Edition::Java => {
                let version = self.version;
                generated_packets::deserialize_java_serverbound_packet(
                    self.state,
                    self.packet_id,
                    &self.payload,
                    version,
                )
                .map_or(ServerboundPacket::Unknown, ServerboundPacket::Java)
            }
            Edition::Bedrock => generated_packets::deserialize_bedrock_serverbound_packet(
                self.packet_id,
                &self.payload,
            )
            .map_or(ServerboundPacket::Unknown, ServerboundPacket::Bedrock),
        };

        Event::PacketReceivedEvent(PacketReceivedEventData {
            user,
            state: to_wasm_state(self.state),
            protocol_version: self.protocol_version,
            packet,
            replacement: None,
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
            if !self.cancelled
                && let Some(replacement) = data.replacement
            {
                let bytes = match replacement {
                    ServerboundPacket::Java(packet) if self.user.edition == Edition::Java => {
                        generated_packets::serialize_java_serverbound_packet(
                            &packet,
                            self.state,
                            self.version,
                        )
                    }
                    ServerboundPacket::Bedrock(packet) if self.user.edition == Edition::Bedrock => {
                        generated_packets::serialize_bedrock_serverbound_packet(&packet)
                    }
                    _ => None,
                };
                if let Some(packet) = bytes.and_then(|bytes| decode_packet(bytes).ok()) {
                    self.packet_id = packet.id;
                    self.payload = packet.payload;
                } else {
                    self.cancelled = true;
                    tracing::warn!("Invalid received packet replacement");
                }
            }
        }
    }
    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        let Event::PacketReceivedEvent(data) = &event else {
            panic!("unexpected event type")
        };
        let user = state
            .packet_user(&data.user)
            .expect("invalid user resource");
        let mut result = Self::new(user, data.packet_id, bytes::Bytes::new());
        result.apply_wasm_event(event, state);
        result
    }
}

impl ToFromWasmEvent for PacketSentEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let user = state
            .add_user(self.user.clone())
            .expect("failed to add user resource");

        let packet = match self.user.edition {
            Edition::Java => generated_packets::deserialize_java_clientbound_packet(
                self.state,
                self.packet_id,
                &self.payload,
                self.version,
            )
            .map_or(ClientboundPacket::Unknown, ClientboundPacket::Java),
            Edition::Bedrock => ClientboundPacket::Unknown,
        };

        Event::PacketSentEvent(PacketSentEventData {
            user,
            state: to_wasm_state(self.state),
            protocol_version: self.protocol_version,
            packet,
            replacement: None,
            packet_id: self.packet_id,
            raw_payload: self.payload.iter().copied().collect(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::PacketSentEvent(data) = event {
            self.packet_id = data.packet_id;
            self.payload = data.raw_payload.into();
            self.cancelled = data.cancelled;
            if !self.cancelled
                && let Some(replacement) = data.replacement
            {
                let bytes = match replacement {
                    ClientboundPacket::Java(packet) if self.user.edition == Edition::Java => {
                        generated_packets::serialize_java_clientbound_packet(
                            &packet,
                            self.state,
                            self.version,
                        )
                    }
                    ClientboundPacket::Bedrock(packet) if self.user.edition == Edition::Bedrock => {
                        generated_packets::serialize_bedrock_packet(&packet)
                    }
                    _ => None,
                };
                if let Some(packet) = bytes.and_then(|bytes| decode_packet(bytes).ok()) {
                    self.packet_id = packet.id;
                    self.payload = packet.payload;
                } else {
                    self.cancelled = true;
                    tracing::warn!("Invalid sent packet replacement");
                }
            }
        }
    }
    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        let Event::PacketSentEvent(data) = &event else {
            panic!("unexpected event type")
        };
        let user = state
            .packet_user(&data.user)
            .expect("invalid user resource");
        let mut result = Self::new(user, data.packet_id, bytes::Bytes::new());
        result.apply_wasm_event(event, state);
        result
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

#[cfg(test)]
mod packet_tests {
    use super::*;
    use crate::{
        net::user::User,
        plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::{
            java_packets as packets, user::ConnectionState as WasmConnectionState,
        },
    };
    use bytes::Bytes;
    use pumpkin_protocol::ConnectionState;
    use pumpkin_util::version::JavaMinecraftVersion;
    use std::sync::{Arc, Weak};
    use wasmtime::component::Resource;

    fn java_user(receive: ConnectionState, send: ConnectionState) -> Arc<User> {
        let user = User::new(
            "127.0.0.1:25565".parse().unwrap(),
            Edition::Java,
            Weak::new(),
        );
        user.decoder_state.store(receive);
        user.encoder_state.store(send);
        user.java_version.store(JavaMinecraftVersion::V_1_21_11);
        user.protocol_version
            .store(Some(JavaMinecraftVersion::V_1_21_11.protocol_version()));
        user
    }

    #[test]
    fn packet_events_apply_raw_changes_and_consume_users() {
        for cancelled in [false, true] {
            let user = java_user(ConnectionState::Login, ConnectionState::Config);
            let mut state = PluginHostState::new();
            let mut received =
                PacketReceivedEvent::new(user.clone(), 32767, Bytes::from_static(&[255, 0]));
            let mut sent = PacketSentEvent::new(user.clone(), 32767, Bytes::from_static(&[128, 0]));
            user.decoder_state.store(ConnectionState::Play);
            user.encoder_state.store(ConnectionState::Play);
            user.protocol_version.store(Some(47));

            let Event::PacketReceivedEvent(mut data) = received.to_wasm_event(&mut state) else {
                panic!("received event expected")
            };
            assert!(matches!(data.state, WasmConnectionState::Login));
            assert_eq!(
                data.protocol_version,
                Some(JavaMinecraftVersion::V_1_21_11.protocol_version())
            );
            assert!(matches!(data.packet, ServerboundPacket::Unknown));
            assert!(Arc::ptr_eq(&user, &state.packet_user(&data.user).unwrap()));
            let user_rep = data.user.rep();
            data.packet_id = 17;
            data.raw_payload = vec![9, 8, 7];
            data.cancelled = cancelled;
            received.apply_wasm_event(Event::PacketReceivedEvent(data), &mut state);
            assert_eq!(received.packet_id, 17);
            assert_eq!(received.payload.as_ref(), &[9, 8, 7]);
            assert_eq!(received.cancelled, cancelled);
            assert!(state.packet_user(&Resource::new_own(user_rep)).is_err());

            let Event::PacketSentEvent(mut data) = sent.to_wasm_event(&mut state) else {
                panic!("sent event expected")
            };
            assert!(matches!(data.state, WasmConnectionState::Config));
            assert_eq!(
                data.protocol_version,
                Some(JavaMinecraftVersion::V_1_21_11.protocol_version())
            );
            assert!(matches!(data.packet, ClientboundPacket::Unknown));
            assert!(Arc::ptr_eq(&user, &state.packet_user(&data.user).unwrap()));
            let user_rep = data.user.rep();
            data.packet_id = 19;
            data.raw_payload = vec![6, 5, 4];
            data.cancelled = cancelled;
            sent.apply_wasm_event(Event::PacketSentEvent(data), &mut state);
            assert_eq!(sent.packet_id, 19);
            assert_eq!(sent.payload.as_ref(), &[6, 5, 4]);
            assert_eq!(sent.cancelled, cancelled);
            assert!(state.packet_user(&Resource::new_own(user_rep)).is_err());
        }
    }

    #[test]
    fn packet_events_apply_typed_replacements_and_reject_invalid_ones() {
        let user = java_user(ConnectionState::Status, ConnectionState::Status);
        let mut state = PluginHostState::new();
        let mut received = PacketReceivedEvent::new(user.clone(), 0, Bytes::new());
        let Event::PacketReceivedEvent(mut data) = received.to_wasm_event(&mut state) else {
            panic!("received event expected")
        };
        let user_rep = data.user.rep();
        data.replacement = Some(ServerboundPacket::Java(
            packets::ServerboundPacket::StatusSStatusPingRequest(
                packets::StatusSStatusPingRequest {
                    payload: 0x0102030405060708,
                },
            ),
        ));
        received.apply_wasm_event(Event::PacketReceivedEvent(data), &mut state);
        assert!(!received.cancelled);
        assert_eq!(received.packet_id, 1);
        assert_eq!(received.payload.as_ref(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(state.packet_user(&Resource::new_own(user_rep)).is_err());

        let mut sent = PacketSentEvent::new(user, 0, Bytes::from_static(b"\x02{}"));
        let Event::PacketSentEvent(mut data) = sent.to_wasm_event(&mut state) else {
            panic!("sent event expected")
        };
        let user_rep = data.user.rep();
        data.replacement = Some(ClientboundPacket::Java(
            packets::ClientboundPacket::StatusCPingResponse(packets::StatusCPingResponse {
                payload: -1,
            }),
        ));
        sent.apply_wasm_event(Event::PacketSentEvent(data), &mut state);
        assert!(!sent.cancelled);
        assert_eq!(sent.packet_id, 1);
        assert_eq!(sent.payload.as_ref(), &[255; 8]);
        assert!(state.packet_user(&Resource::new_own(user_rep)).is_err());

        let Event::PacketReceivedEvent(mut data) = received.to_wasm_event(&mut state) else {
            panic!("received event expected")
        };
        data.replacement = Some(ServerboundPacket::Java(
            packets::ServerboundPacket::LoginSLoginAcknowledged,
        ));
        received.apply_wasm_event(Event::PacketReceivedEvent(data), &mut state);
        assert!(received.cancelled);
        let Event::PacketSentEvent(mut data) = sent.to_wasm_event(&mut state) else {
            panic!("sent event expected")
        };
        data.replacement = Some(ClientboundPacket::Java(
            packets::ClientboundPacket::ConfigCFinishConfig,
        ));
        sent.apply_wasm_event(Event::PacketSentEvent(data), &mut state);
        assert!(sent.cancelled);
    }

    #[test]
    fn packet_read_views_preserve_original_and_unknown_payloads() {
        let user = java_user(ConnectionState::Login, ConnectionState::Status);
        let mut state = PluginHostState::new();
        let mut login = vec![0x86, 0];
        login.extend_from_slice(b"tester");
        login.extend_from_slice(&[1; 16]);
        let login = Bytes::from(login);
        let mut received = PacketReceivedEvent::new(user.clone(), 0, login.clone());
        let mut sent = PacketSentEvent::new(user.clone(), 0, Bytes::from_static(b"\x82\x00{}"));
        user.java_version.store(JavaMinecraftVersion::V_1_8);
        let returned = received.to_wasm_event(&mut state);
        assert!(
            matches!(&returned, Event::PacketReceivedEvent(data) if matches!(&data.packet, ServerboundPacket::Java(packets::ServerboundPacket::LoginSLoginStart(_))))
        );
        received.apply_wasm_event(returned, &mut state);
        assert_eq!(received.payload, login);
        assert!(!received.cancelled);
        let returned = sent.to_wasm_event(&mut state);
        assert!(
            matches!(&returned, Event::PacketSentEvent(data) if matches!(&data.packet, ClientboundPacket::Java(packets::ClientboundPacket::StatusCStatusResponse(_))))
        );
        sent.apply_wasm_event(returned, &mut state);
        assert_eq!(sent.payload.as_ref(), b"\x82\x00{}");
        assert!(!sent.cancelled);

        let payload = Bytes::from_static(&[0, 255, 128, 0]);
        let mut received = PacketReceivedEvent::new(user.clone(), 32767, payload.clone());
        let returned = received.to_wasm_event(&mut state);
        assert!(
            matches!(&returned, Event::PacketReceivedEvent(data) if matches!(data.packet, ServerboundPacket::Unknown))
        );
        received.apply_wasm_event(returned, &mut state);
        assert_eq!(received.packet_id, 32767);
        assert_eq!(received.payload, payload);
        assert!(!received.cancelled);
        let mut sent = PacketSentEvent::new(user, 32767, payload.clone());
        let returned = sent.to_wasm_event(&mut state);
        assert!(
            matches!(&returned, Event::PacketSentEvent(data) if matches!(data.packet, ClientboundPacket::Unknown))
        );
        sent.apply_wasm_event(returned, &mut state);
        assert_eq!(sent.packet_id, 32767);
        assert_eq!(sent.payload, payload);
        assert!(!sent.cancelled);
    }
}
