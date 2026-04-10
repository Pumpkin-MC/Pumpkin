use crate::{
    Context, Result, Server,
    events::{EventHandler, EventPriority, FromIntoEvent, player::PlayerCustomPayloadEvent},
    wit::pumpkin::plugin::event::PlayerCustomPayloadEventData,
};

pub use crate::wit::pumpkin::plugin::event::RawPacketEventData;
pub use crate::wit::pumpkin::plugin::packet::{
    BedrockState, ConnectionState, JavaState, Packet, PacketDirection, RawPacket,
};

/// A lightweight packet event wrapper for custom payload packets.
pub struct PacketEvent<'a> {
    pub player: &'a crate::wit::pumpkin::plugin::player::Player,
    pub packet: Packet,
}

/// Handles incoming custom payload packets.
pub trait PacketHandler: Send + Sync {
    fn handle(&self, server: Server, event: PacketEvent<'_>);
}

struct PacketHandlerWrapper<H> {
    handler: H,
    channel: Option<String>,
}

impl<H: PacketHandler + Send + Sync> EventHandler<PlayerCustomPayloadEvent>
    for PacketHandlerWrapper<H>
{
    fn handle(
        &self,
        server: Server,
        event: PlayerCustomPayloadEventData,
    ) -> PlayerCustomPayloadEventData {
        if let Some(channel) = &self.channel
            && event.channel != *channel
        {
            return event;
        }

        let packet = Packet {
            channel: event.channel.clone(),
            data: event.data.clone(),
        };
        let packet_event = PacketEvent {
            player: &event.player,
            packet,
        };

        self.handler.handle(server, packet_event);
        event
    }
}

impl Packet {
    /// Creates a new custom payload packet.
    pub fn new(channel: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            channel: channel.into(),
            data: data.into(),
        }
    }
}

impl RawPacket {
    /// Creates a new raw packet with the given id and payload.
    pub fn new(id: i32, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            payload: payload.into(),
        }
    }
}

impl crate::wit::pumpkin::plugin::player::Player {
    /// Sends a raw packet by id + payload convenience.
    pub fn send_raw_packet_id(&self, id: i32, payload: impl Into<Vec<u8>>) {
        let packet = RawPacket::new(id, payload);
        self.send_raw_packet(&packet);
    }

    /// Sends a custom payload packet on the given channel.
    pub fn send_packet_on_channel(&self, channel: impl Into<String>, data: impl Into<Vec<u8>>) {
        let packet = Packet::new(channel, data);
        self.send_packet(&packet);
    }
}

impl crate::wit::pumpkin::plugin::context::Server {
    /// Broadcasts a raw packet by id + payload convenience.
    pub fn broadcast_raw_packet_id(&self, id: i32, payload: impl Into<Vec<u8>>) {
        let packet = RawPacket::new(id, payload);
        self.broadcast_raw_packet(&packet);
    }

    /// Broadcasts a custom payload packet on the given channel.
    pub fn broadcast_packet_on_channel(
        &self,
        channel: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) {
        let packet = Packet::new(channel, data);
        self.broadcast_packet(&packet);
    }
}

/// Marker type for raw packet events.
pub struct RawPacketEvent;

impl FromIntoEvent for RawPacketEvent {
    const EVENT_TYPE: crate::wit::pumpkin::plugin::event::EventType =
        crate::wit::pumpkin::plugin::event::EventType::RawPacketEvent;
    type Data = RawPacketEventData;

    fn data_from_event(event: crate::wit::pumpkin::plugin::event::Event) -> Self::Data {
        match event {
            crate::wit::pumpkin::plugin::event::Event::RawPacketEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> crate::wit::pumpkin::plugin::event::Event {
        crate::wit::pumpkin::plugin::event::Event::RawPacketEvent(data)
    }
}

/// Filters raw packet events before they reach the handler.
#[derive(Clone, Default)]
pub struct RawPacketFilter {
    pub direction: Option<PacketDirection>,
    pub state: Option<ConnectionState>,
    pub packet_id: Option<i32>,
}

/// Handles incoming raw packet events.
pub trait RawPacketHandler: Send + Sync {
    fn handle(&self, server: Server, event: RawPacketEventData) -> RawPacketEventData;
}

struct RawPacketHandlerWrapper<H> {
    handler: H,
    filter: RawPacketFilter,
}

impl<H: RawPacketHandler + Send + Sync> EventHandler<RawPacketEvent>
    for RawPacketHandlerWrapper<H>
{
    fn handle(&self, server: Server, event: RawPacketEventData) -> RawPacketEventData {
        if let Some(direction) = self.filter.direction
            && event.packet.direction != direction
        {
            return event;
        }

        if let Some(state) = &self.filter.state
            && !connection_state_eq(&event.packet.state, state)
        {
            return event;
        }

        if let Some(packet_id) = self.filter.packet_id
            && event.packet.packet.id != packet_id
        {
            return event;
        }

        self.handler.handle(server, event)
    }
}

fn connection_state_eq(a: &ConnectionState, b: &ConnectionState) -> bool {
    match (a, b) {
        (ConnectionState::Java(a), ConnectionState::Java(b)) => matches!(
            (a, b),
            (JavaState::Handshake, JavaState::Handshake)
                | (JavaState::Status, JavaState::Status)
                | (JavaState::Login, JavaState::Login)
                | (JavaState::Config, JavaState::Config)
                | (JavaState::Play, JavaState::Play)
                | (JavaState::Transfer, JavaState::Transfer)
        ),
        (ConnectionState::Bedrock(a), ConnectionState::Bedrock(b)) => matches!(
            (a, b),
            (BedrockState::Offline, BedrockState::Offline)
                | (BedrockState::Raknet, BedrockState::Raknet)
                | (BedrockState::Game, BedrockState::Game)
        ),
        _ => false,
    }
}

impl Context {
    /// Registers a handler for all incoming custom payload packets.
    pub fn register_packet_handler<H: PacketHandler + Send + Sync + 'static>(
        &self,
        handler: H,
        event_priority: EventPriority,
    ) -> Result<u32> {
        let wrapper = PacketHandlerWrapper {
            handler,
            channel: None,
        };
        self.register_event_handler::<PlayerCustomPayloadEvent, _>(wrapper, event_priority, false)
    }

    /// Registers a handler for incoming custom payload packets on a specific channel.
    pub fn register_packet_handler_for_channel<H: PacketHandler + Send + Sync + 'static>(
        &self,
        channel: impl Into<String>,
        handler: H,
        event_priority: EventPriority,
    ) -> Result<u32> {
        let wrapper = PacketHandlerWrapper {
            handler,
            channel: Some(channel.into()),
        };
        self.register_event_handler::<PlayerCustomPayloadEvent, _>(wrapper, event_priority, false)
    }

    /// Registers a handler for all raw packet events.
    pub fn register_raw_packet_handler<H: RawPacketHandler + Send + Sync + 'static>(
        &self,
        handler: H,
        event_priority: EventPriority,
        blocking: bool,
    ) -> Result<u32> {
        self.register_raw_packet_handler_with_filter(
            RawPacketFilter::default(),
            handler,
            event_priority,
            blocking,
        )
    }

    /// Registers a handler for raw packet events with a filter.
    pub fn register_raw_packet_handler_with_filter<H: RawPacketHandler + Send + Sync + 'static>(
        &self,
        filter: RawPacketFilter,
        handler: H,
        event_priority: EventPriority,
        blocking: bool,
    ) -> Result<u32> {
        let wrapper = RawPacketHandlerWrapper { handler, filter };
        self.register_event_handler::<RawPacketEvent, _>(wrapper, event_priority, blocking)
    }
}

/// Small packet codec helpers for raw packet parsing and construction.
pub mod codec {
    use std::{string::String, vec, vec::Vec};

    /// Read errors when decoding a packet payload.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum PacketReadError {
        UnexpectedEof,
        VarIntTooLong,
        StringTooLong,
        InvalidUtf8,
    }

    /// A lightweight reader over a packet payload.
    pub struct PacketReader<'a> {
        buf: &'a [u8],
        pos: usize,
    }

    impl<'a> PacketReader<'a> {
        /// Creates a new reader over the given payload bytes.
        pub fn new(buf: &'a [u8]) -> Self {
            Self { buf, pos: 0 }
        }

        /// Returns how many bytes are still unread.
        pub fn remaining(&self) -> usize {
            self.buf.len().saturating_sub(self.pos)
        }

        /// Returns the current cursor position.
        pub fn position(&self) -> usize {
            self.pos
        }

        fn read_exact(&mut self, len: usize) -> Result<&'a [u8], PacketReadError> {
            let end = self
                .pos
                .checked_add(len)
                .ok_or(PacketReadError::UnexpectedEof)?;
            let slice = self
                .buf
                .get(self.pos..end)
                .ok_or(PacketReadError::UnexpectedEof)?;
            self.pos = end;
            Ok(slice)
        }

        /// Reads a single byte.
        pub fn read_u8(&mut self) -> Result<u8, PacketReadError> {
            Ok(*self.read_exact(1)?.first().unwrap())
        }

        /// Reads a boolean (0 = false, otherwise true).
        pub fn read_bool(&mut self) -> Result<bool, PacketReadError> {
            Ok(self.read_u8()? != 0)
        }

        /// Reads a big-endian u16.
        pub fn read_u16(&mut self) -> Result<u16, PacketReadError> {
            let bytes = self.read_exact(2)?;
            Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
        }

        /// Reads a big-endian i32.
        pub fn read_i32(&mut self) -> Result<i32, PacketReadError> {
            let bytes = self.read_exact(4)?;
            Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }

        /// Reads a big-endian i64.
        pub fn read_i64(&mut self) -> Result<i64, PacketReadError> {
            let bytes = self.read_exact(8)?;
            Ok(i64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }

        /// Reads a VarInt.
        pub fn read_varint(&mut self) -> Result<i32, PacketReadError> {
            let mut num_read = 0u32;
            let mut result: i32 = 0;

            loop {
                let byte = self.read_u8()?;
                let value = i32::from(byte & 0x7F);
                result |= value << (7 * num_read);
                num_read += 1;
                if num_read > 5 {
                    return Err(PacketReadError::VarIntTooLong);
                }
                if (byte & 0x80) == 0 {
                    return Ok(result);
                }
            }
        }

        /// Reads a VarLong.
        pub fn read_varlong(&mut self) -> Result<i64, PacketReadError> {
            let mut num_read = 0u32;
            let mut result: i64 = 0;

            loop {
                let byte = self.read_u8()?;
                let value = i64::from(byte & 0x7F);
                result |= value << (7 * num_read);
                num_read += 1;
                if num_read > 10 {
                    return Err(PacketReadError::VarIntTooLong);
                }
                if (byte & 0x80) == 0 {
                    return Ok(result);
                }
            }
        }

        /// Reads a byte slice of length `len`.
        pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], PacketReadError> {
            self.read_exact(len)
        }

        /// Reads a UUID as 16 raw bytes.
        pub fn read_uuid_bytes(&mut self) -> Result<[u8; 16], PacketReadError> {
            let bytes = self.read_exact(16)?;
            Ok([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ])
        }

        /// Reads a length-prefixed UTF-8 string.
        /// `max_len` is the maximum allowed character length to guard against abuse.
        pub fn read_string(&mut self, max_len: usize) -> Result<String, PacketReadError> {
            let len =
                usize::try_from(self.read_varint()?).map_err(|_| PacketReadError::StringTooLong)?;
            if len > max_len {
                return Err(PacketReadError::StringTooLong);
            }
            let bytes = self.read_exact(len)?;
            core::str::from_utf8(bytes)
                .map(|s| s.to_string())
                .map_err(|_| PacketReadError::InvalidUtf8)
        }
    }

    /// A lightweight writer to build packet payloads.
    pub struct PacketWriter {
        buf: Vec<u8>,
    }

    impl PacketWriter {
        /// Creates an empty writer.
        pub fn new() -> Self {
            Self { buf: vec![] }
        }

        /// Returns the underlying buffer.
        pub fn into_inner(self) -> Vec<u8> {
            self.buf
        }

        /// Writes a single byte.
        pub fn write_u8(&mut self, value: u8) {
            self.buf.push(value);
        }

        /// Writes a boolean as 0/1.
        pub fn write_bool(&mut self, value: bool) {
            self.buf.push(if value { 1 } else { 0 });
        }

        /// Writes a big-endian u16.
        pub fn write_u16(&mut self, value: u16) {
            self.buf.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a big-endian i32.
        pub fn write_i32(&mut self, value: i32) {
            self.buf.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a big-endian i64.
        pub fn write_i64(&mut self, value: i64) {
            self.buf.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a VarInt.
        pub fn write_varint(&mut self, mut value: i32) {
            loop {
                let mut temp = (value & 0x7F) as u8;
                value = ((value as u32) >> 7) as i32;
                if value != 0 {
                    temp |= 0x80;
                }
                self.buf.push(temp);
                if value == 0 {
                    break;
                }
            }
        }

        /// Writes a VarLong.
        pub fn write_varlong(&mut self, mut value: i64) {
            loop {
                let mut temp = (value & 0x7F) as u8;
                value = ((value as u64) >> 7) as i64;
                if value != 0 {
                    temp |= 0x80;
                }
                self.buf.push(temp);
                if value == 0 {
                    break;
                }
            }
        }

        /// Writes a byte slice.
        pub fn write_bytes(&mut self, bytes: &[u8]) {
            self.buf.extend_from_slice(bytes);
        }

        /// Writes a UUID as 16 raw bytes.
        pub fn write_uuid_bytes(&mut self, bytes: &[u8; 16]) {
            self.buf.extend_from_slice(bytes);
        }

        /// Writes a length-prefixed UTF-8 string.
        pub fn write_string(&mut self, value: &str) {
            self.write_varint(value.len() as i32);
            self.buf.extend_from_slice(value.as_bytes());
        }
    }

    impl Default for PacketWriter {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Convenience wrapper for raw packet events with version-aware helpers.
pub struct PacketWrapper {
    event: RawPacketEventData,
    protocol_version: Option<u32>,
}

impl PacketWrapper {
    /// Builds a wrapper from a raw packet event.
    pub fn new(event: RawPacketEventData) -> Self {
        let protocol_version = event
            .player
            .as_ref()
            .map(|player| player.get_protocol_version());
        Self {
            event,
            protocol_version,
        }
    }

    /// Returns the underlying event data.
    pub fn event(&self) -> &RawPacketEventData {
        &self.event
    }

    /// Returns a mutable reference to the underlying event data.
    pub fn event_mut(&mut self) -> &mut RawPacketEventData {
        &mut self.event
    }

    /// Consumes the wrapper and returns the underlying event data.
    pub fn into_event(self) -> RawPacketEventData {
        self.event
    }

    /// Returns the packet id.
    pub fn id(&self) -> i32 {
        self.event.packet.packet.id
    }

    /// Returns the packet direction.
    pub fn direction(&self) -> PacketDirection {
        self.event.packet.direction
    }

    /// Returns the connection state.
    pub fn state(&self) -> &ConnectionState {
        &self.event.packet.state
    }

    /// Returns `true` if this is a Java connection state.
    pub fn is_java(&self) -> bool {
        matches!(self.event.packet.state, ConnectionState::Java(_))
    }

    /// Returns `true` if this is a Bedrock connection state.
    pub fn is_bedrock(&self) -> bool {
        matches!(self.event.packet.state, ConnectionState::Bedrock(_))
    }

    /// Returns the protocol version number for this player, if available.
    pub fn protocol_version(&self) -> Option<u32> {
        self.protocol_version
    }

    /// Returns the resolved Java Minecraft version, if this is a Java client.
    pub fn minecraft_version(&self) -> Option<pumpkin_util::version::MinecraftVersion> {
        let protocol = self.protocol_version?;
        Some(pumpkin_util::version::MinecraftVersion::from_protocol(
            protocol,
        ))
    }

    /// Resolves a packet id for the current protocol version.
    pub fn resolve_packet_id(&self, packet_id: &crate::packet_ids_full::PacketId) -> Option<i32> {
        self.minecraft_version()
            .map(|version| packet_id.to_id(version))
    }

    /// Resolves a Java packet id by `phase` and `name` for this player's protocol version.
    pub fn java_packet_id(&self, phase: &str, name: &str) -> Option<i32> {
        let version = self.minecraft_version()?;
        java::catalog::java_packet_catalog().get_id(version, self.direction(), phase, name)
    }

    /// Resolves a Java packet `(phase, name)` by raw packet id for this player's protocol version.
    pub fn java_packet_name(&self) -> Option<java::catalog::JavaPacketName> {
        let version = self.minecraft_version()?;
        java::catalog::java_packet_catalog().get_name(version, self.direction(), self.id())
    }

    /// Returns the packet payload bytes.
    pub fn payload(&self) -> &[u8] {
        &self.event.packet.packet.payload
    }

    /// Returns a mutable view of the packet payload bytes.
    pub fn payload_mut(&mut self) -> &mut Vec<u8> {
        &mut self.event.packet.packet.payload
    }

    /// Creates a reader over the current payload.
    pub fn reader(&self) -> codec::PacketReader<'_> {
        codec::PacketReader::new(self.payload())
    }

    /// Replaces the payload bytes.
    pub fn replace_payload(&mut self, payload: Vec<u8>) {
        self.event.packet.packet.payload = payload;
    }

    /// Marks the packet event as cancelled.
    pub fn cancel(&mut self) {
        self.event.packet.cancelled = true;
    }
}

/// Java edition packet helpers.
pub mod java {
    use std::{collections::BTreeMap, sync::OnceLock};

    use super::{
        PacketDirection, RawPacket,
        codec::{PacketReadError, PacketReader, PacketWriter},
    };
    use pumpkin_util::version::MinecraftVersion;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct VersionPacketAsset {
        #[allow(dead_code)]
        version: u32,
        serverbound: BTreeMap<String, BTreeMap<String, i32>>,
        clientbound: BTreeMap<String, BTreeMap<String, i32>>,
    }

    /// A packet descriptor exposed by the Java packet catalog.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct JavaPacketDescriptor {
        pub phase: String,
        pub name: String,
        pub id: i32,
    }

    /// A resolved packet name pair `(phase, name)`.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct JavaPacketName {
        pub phase: String,
        pub name: String,
    }

    #[derive(Debug, Default)]
    struct VersionPackets {
        serverbound: BTreeMap<String, BTreeMap<String, i32>>,
        clientbound: BTreeMap<String, BTreeMap<String, i32>>,
        reverse_serverbound: BTreeMap<i32, JavaPacketName>,
        reverse_clientbound: BTreeMap<i32, JavaPacketName>,
    }

    /// Version-aware catalog for all Java packet ids shipped with Pumpkin.
    #[derive(Debug, Default)]
    pub struct JavaPacketCatalog {
        versions: BTreeMap<MinecraftVersion, VersionPackets>,
    }

    impl JavaPacketCatalog {
        fn load_embedded() -> Self {
            let mut catalog = Self::default();

            for (version, json) in [
                (
                    MinecraftVersion::V_1_21,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_2,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_2_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_4,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_4_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_5,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_5_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_6,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_6_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_7,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_7_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_9,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_9_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_1_21_11,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/1_21_11_packets.json"
                    )),
                ),
                (
                    MinecraftVersion::V_26_1,
                    include_str!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../assets/packet/26_1_packets.json"
                    )),
                ),
            ] {
                let parsed: VersionPacketAsset = serde_json::from_str(json).unwrap_or_else(|err| {
                    panic!("failed to parse embedded packet asset for {version}: {err}")
                });

                let mut version_packets = VersionPackets {
                    serverbound: parsed.serverbound,
                    clientbound: parsed.clientbound,
                    reverse_serverbound: BTreeMap::new(),
                    reverse_clientbound: BTreeMap::new(),
                };

                for (phase, packets) in &version_packets.serverbound {
                    for (name, id) in packets {
                        version_packets.reverse_serverbound.insert(
                            *id,
                            JavaPacketName {
                                phase: phase.clone(),
                                name: name.clone(),
                            },
                        );
                    }
                }

                for (phase, packets) in &version_packets.clientbound {
                    for (name, id) in packets {
                        version_packets.reverse_clientbound.insert(
                            *id,
                            JavaPacketName {
                                phase: phase.clone(),
                                name: name.clone(),
                            },
                        );
                    }
                }

                catalog.versions.insert(version, version_packets);
            }

            catalog
        }

        fn direction_map(
            packets: &VersionPackets,
            direction: PacketDirection,
        ) -> &BTreeMap<String, BTreeMap<String, i32>> {
            match direction {
                PacketDirection::Serverbound => &packets.serverbound,
                PacketDirection::Clientbound => &packets.clientbound,
            }
        }

        fn reverse_direction_map(
            packets: &VersionPackets,
            direction: PacketDirection,
        ) -> &BTreeMap<i32, JavaPacketName> {
            match direction {
                PacketDirection::Serverbound => &packets.reverse_serverbound,
                PacketDirection::Clientbound => &packets.reverse_clientbound,
            }
        }

        /// Resolves a packet id for `(version, direction, phase, name)`.
        pub fn get_id(
            &self,
            version: MinecraftVersion,
            direction: PacketDirection,
            phase: &str,
            name: &str,
        ) -> Option<i32> {
            let packets = self.versions.get(&version)?;
            let phase_packets = Self::direction_map(packets, direction).get(phase)?;
            phase_packets.get(name).copied()
        }

        /// Resolves packet `(phase, name)` by `(version, direction, id)`.
        pub fn get_name(
            &self,
            version: MinecraftVersion,
            direction: PacketDirection,
            id: i32,
        ) -> Option<JavaPacketName> {
            let packets = self.versions.get(&version)?;
            Self::reverse_direction_map(packets, direction)
                .get(&id)
                .cloned()
        }

        /// Returns every packet descriptor for `(version, direction)`.
        pub fn list(
            &self,
            version: MinecraftVersion,
            direction: PacketDirection,
        ) -> Vec<JavaPacketDescriptor> {
            let Some(packets) = self.versions.get(&version) else {
                return Vec::new();
            };

            Self::direction_map(packets, direction)
                .iter()
                .flat_map(|(phase, names)| {
                    names.iter().map(|(name, id)| JavaPacketDescriptor {
                        phase: phase.clone(),
                        name: name.clone(),
                        id: *id,
                    })
                })
                .collect()
        }
    }

    pub mod catalog {
        use super::OnceLock;
        pub use super::{JavaPacketCatalog, JavaPacketDescriptor, JavaPacketName};

        static JAVA_PACKET_CATALOG: OnceLock<JavaPacketCatalog> = OnceLock::new();

        /// Returns the global Java packet catalog that includes all supported versions.
        pub fn java_packet_catalog() -> &'static JavaPacketCatalog {
            JAVA_PACKET_CATALOG.get_or_init(JavaPacketCatalog::load_embedded)
        }
    }

    /// Parsed form of the serverbound chat message packet (Java, Play state).
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ServerboundChatMessage {
        pub message: String,
        pub timestamp: i64,
        pub salt: i64,
        pub signature: Option<Vec<u8>>,
        pub message_count: i32,
        /// Fixed 20-bit bitset represented as 3 raw bytes.
        pub acknowledged: [u8; 3],
        /// Optional checksum (present in newer protocol revisions).
        pub checksum: Option<u8>,
    }

    impl ServerboundChatMessage {
        /// Decodes a serverbound chat message payload.
        pub fn decode(payload: &[u8]) -> Result<Self, PacketReadError> {
            let mut reader = PacketReader::new(payload);
            let message = reader.read_string(256)?;
            let timestamp = reader.read_i64()?;
            let salt = reader.read_i64()?;

            let has_signature = reader.read_bool()?;
            let signature = if has_signature {
                let len = usize::try_from(reader.read_varint()?)
                    .map_err(|_| PacketReadError::StringTooLong)?;
                if len > 256 {
                    return Err(PacketReadError::StringTooLong);
                }
                Some(reader.read_bytes(len)?.to_vec())
            } else {
                None
            };

            let message_count = reader.read_varint()?;
            let ack_bytes = reader.read_bytes(3)?;
            let acknowledged = [ack_bytes[0], ack_bytes[1], ack_bytes[2]];

            let checksum = if reader.remaining() >= 1 {
                Some(reader.read_u8()?)
            } else {
                None
            };

            Ok(Self {
                message,
                timestamp,
                salt,
                signature,
                message_count,
                acknowledged,
                checksum,
            })
        }

        /// Encodes this message into a packet payload.
        pub fn encode(&self) -> Vec<u8> {
            let mut writer = PacketWriter::new();
            writer.write_string(&self.message);
            writer.write_i64(self.timestamp);
            writer.write_i64(self.salt);

            match &self.signature {
                Some(sig) => {
                    writer.write_bool(true);
                    writer.write_varint(sig.len() as i32);
                    writer.write_bytes(sig);
                }
                None => {
                    writer.write_bool(false);
                }
            }

            writer.write_varint(self.message_count);
            writer.write_bytes(&self.acknowledged);

            if let Some(checksum) = self.checksum {
                writer.write_u8(checksum);
            }

            writer.into_inner()
        }

        /// Builds a raw packet with the provided packet id.
        pub fn to_raw_packet(&self, id: i32) -> RawPacket {
            RawPacket::new(id, self.encode())
        }
    }
}
