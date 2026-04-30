use crate::{
    Context, Result, Server,
    events::{EventHandler, EventPriority, FromIntoEvent, player::PlayerCustomPayloadEvent},
    wit::pumpkin::plugin::event::PlayerCustomPayloadEventData,
};

pub use crate::wit::pumpkin::plugin::event::RawPacketEventData;
pub use crate::wit::pumpkin::plugin::packet::{
    BedrockState, BlockPos, ConnectionState, FieldValue, JavaPacketKey, JavaState, NamedFieldValue,
    Packet, PacketDirection, PacketHandle, PacketReadError, PacketReader as WitPacketReader,
    PacketSchemaError, PacketWriter as WitPacketWriter, RawPacket, Vec3F32, Vec3F64, Vec3I16,
    Vec3I32,
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
    /// Sends a system message to this player.
    ///
    /// When called while a WASM event is being processed, the host may defer the
    /// actual send until the current event returns to avoid recursive packet hooks.
    pub fn send_system_message_text(
        &self,
        text: impl Into<crate::text::TextComponent>,
        overlay: bool,
    ) {
        let text = text.into();
        self.send_system_message(text, overlay);
    }

    /// Sends a raw packet by id + payload convenience.
    ///
    /// When called while a WASM event is being processed, the host may defer the
    /// actual send until the current event returns to avoid recursive packet hooks.
    pub fn send_raw_packet_id(&self, id: i32, payload: impl Into<Vec<u8>>) {
        let packet = RawPacket::new(id, payload);
        self.send_raw_packet(&packet);
    }

    /// Sends a custom payload packet on the given channel.
    ///
    /// When called while a WASM event is being processed, the host may defer the
    /// actual send until the current event returns to avoid recursive packet hooks.
    pub fn send_packet_on_channel(&self, channel: impl Into<String>, data: impl Into<Vec<u8>>) {
        let packet = Packet::new(channel, data);
        self.send_packet(&packet);
    }
}

impl crate::wit::pumpkin::plugin::context::Server {
    /// Broadcasts a raw packet by id + payload convenience.
    ///
    /// When called while a WASM event is being processed, the host may defer the
    /// actual send until the current event returns to avoid recursive packet hooks.
    pub fn broadcast_raw_packet_id(&self, id: i32, payload: impl Into<Vec<u8>>) {
        let packet = RawPacket::new(id, payload);
        self.broadcast_raw_packet(&packet);
    }

    /// Broadcasts a custom payload packet on the given channel.
    ///
    /// When called while a WASM event is being processed, the host may defer the
    /// actual send until the current event returns to avoid recursive packet hooks.
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
            && event.packet.direction() != direction
        {
            return event;
        }

        if let Some(state) = &self.filter.state
            && !connection_state_eq(&event.packet.state(), state)
        {
            return event;
        }

        if let Some(packet_id) = self.filter.packet_id
            && event.packet.id() != packet_id
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

        /// Reads a single signed byte.
        pub fn read_i8(&mut self) -> Result<i8, PacketReadError> {
            Ok(self.read_u8()? as i8)
        }

        /// Reads a big-endian i16.
        pub fn read_i16(&mut self) -> Result<i16, PacketReadError> {
            let bytes = self.read_exact(2)?;
            Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
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

        /// Reads a big-endian f32.
        pub fn read_f32(&mut self) -> Result<f32, PacketReadError> {
            let bytes = self.read_exact(4)?;
            Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        }

        /// Reads a big-endian f64.
        pub fn read_f64(&mut self) -> Result<f64, PacketReadError> {
            let bytes = self.read_exact(8)?;
            Ok(f64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]))
        }

        /// Reads a packed Java `BlockPos`.
        pub fn read_block_pos(&mut self) -> Result<super::BlockPos, PacketReadError> {
            let encoded = self.read_i64()?;
            Ok(super::BlockPos {
                x: (encoded >> 38) as i32,
                y: (encoded << 52 >> 52) as i32,
                z: (encoded << 26 >> 38) as i32,
            })
        }

        /// Reads a 3-component `f32` vector.
        pub fn read_vec3_f32(&mut self) -> Result<super::Vec3F32, PacketReadError> {
            Ok(super::Vec3F32 {
                x: self.read_f32()?,
                y: self.read_f32()?,
                z: self.read_f32()?,
            })
        }

        /// Reads a 3-component `f64` vector.
        pub fn read_vec3_f64(&mut self) -> Result<super::Vec3F64, PacketReadError> {
            Ok(super::Vec3F64 {
                x: self.read_f64()?,
                y: self.read_f64()?,
                z: self.read_f64()?,
            })
        }

        /// Reads a 3-component `i16` vector.
        pub fn read_vec3_i16(&mut self) -> Result<super::Vec3I16, PacketReadError> {
            Ok(super::Vec3I16 {
                x: self.read_i16()?,
                y: self.read_i16()?,
                z: self.read_i16()?,
            })
        }

        /// Reads a 3-component `i32` vector.
        pub fn read_vec3_i32(&mut self) -> Result<super::Vec3I32, PacketReadError> {
            Ok(super::Vec3I32 {
                x: self.read_i32()?,
                y: self.read_i32()?,
                z: self.read_i32()?,
            })
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

        /// Writes a single signed byte.
        pub fn write_i8(&mut self, value: i8) {
            self.buf.push(value as u8);
        }

        /// Writes a big-endian i16.
        pub fn write_i16(&mut self, value: i16) {
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

        /// Writes a big-endian f32.
        pub fn write_f32(&mut self, value: f32) {
            self.buf.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a big-endian f64.
        pub fn write_f64(&mut self, value: f64) {
            self.buf.extend_from_slice(&value.to_be_bytes());
        }

        /// Writes a packed Java `BlockPos`.
        pub fn write_block_pos(&mut self, value: &super::BlockPos) {
            let encoded = ((i64::from(value.x) & 0x3FFFFFF) << 38)
                | ((i64::from(value.z) & 0x3FFFFFF) << 12)
                | (i64::from(value.y) & 0xFFF);
            self.write_i64(encoded);
        }

        /// Writes a 3-component `f32` vector.
        pub fn write_vec3_f32(&mut self, value: &super::Vec3F32) {
            self.write_f32(value.x);
            self.write_f32(value.y);
            self.write_f32(value.z);
        }

        /// Writes a 3-component `f64` vector.
        pub fn write_vec3_f64(&mut self, value: &super::Vec3F64) {
            self.write_f64(value.x);
            self.write_f64(value.y);
            self.write_f64(value.z);
        }

        /// Writes a 3-component `i16` vector.
        pub fn write_vec3_i16(&mut self, value: &super::Vec3I16) {
            self.write_i16(value.x);
            self.write_i16(value.y);
            self.write_i16(value.z);
        }

        /// Writes a 3-component `i32` vector.
        pub fn write_vec3_i32(&mut self, value: &super::Vec3I32) {
            self.write_i32(value.x);
            self.write_i32(value.y);
            self.write_i32(value.z);
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
    packet: PacketHandle,
    protocol_version: Option<u32>,
}

fn to_wit_field_value(value: java::schema::FieldValue) -> FieldValue {
    match value {
        java::schema::FieldValue::U8(v) => FieldValue::U8Value(v),
        java::schema::FieldValue::Bool(v) => FieldValue::BoolValue(v),
        java::schema::FieldValue::U16(v) => FieldValue::U16Value(v),
        java::schema::FieldValue::I8(v) => FieldValue::I8Value(v),
        java::schema::FieldValue::I16(v) => FieldValue::I16Value(v),
        java::schema::FieldValue::I32(v) => FieldValue::I32Value(v),
        java::schema::FieldValue::I64(v) => FieldValue::I64Value(v),
        java::schema::FieldValue::F32(v) => FieldValue::F32Value(f32::from_bits(v)),
        java::schema::FieldValue::F64(v) => FieldValue::F64Value(f64::from_bits(v)),
        java::schema::FieldValue::BlockPos(v) => FieldValue::BlockPosValue(v),
        java::schema::FieldValue::Vec3F32(v) => FieldValue::Vec3F32Value(v),
        java::schema::FieldValue::Vec3F64(v) => FieldValue::Vec3F64Value(v),
        java::schema::FieldValue::Vec3I16(v) => FieldValue::Vec3I16Value(v),
        java::schema::FieldValue::Vec3I32(v) => FieldValue::Vec3I32Value(v),
        java::schema::FieldValue::VarInt(v) => FieldValue::VarintValue(v),
        java::schema::FieldValue::VarLong(v) => FieldValue::VarlongValue(v),
        java::schema::FieldValue::String(v) => FieldValue::StringValue(v),
        java::schema::FieldValue::Bytes(v) => FieldValue::BytesValue(v),
        java::schema::FieldValue::UuidBytes(v) => FieldValue::UuidBytesValue(v.to_vec()),
    }
}

fn to_wit_packet_schema_error(err: java::schema::PacketSchemaError) -> PacketSchemaError {
    match err {
        java::schema::PacketSchemaError::Decode(codec::PacketReadError::UnexpectedEof) => {
            PacketSchemaError::DecodeUnexpectedEof
        }
        java::schema::PacketSchemaError::Decode(codec::PacketReadError::VarIntTooLong) => {
            PacketSchemaError::DecodeVarintTooLong
        }
        java::schema::PacketSchemaError::Decode(codec::PacketReadError::StringTooLong) => {
            PacketSchemaError::DecodeStringTooLong
        }
        java::schema::PacketSchemaError::Decode(codec::PacketReadError::InvalidUtf8) => {
            PacketSchemaError::DecodeInvalidUtf8
        }
        java::schema::PacketSchemaError::MissingField(_)
        | java::schema::PacketSchemaError::WrongType { .. } => PacketSchemaError::MissingSchema,
    }
}

impl PacketWrapper {
    /// Builds a wrapper from a packet-handle resource.
    pub fn new(packet: PacketHandle) -> Self {
        let protocol_version = packet.protocol_version();
        Self {
            packet,
            protocol_version,
        }
    }

    /// Builds a wrapper from a raw packet event, consuming the event handle.
    pub fn from_event(event: RawPacketEventData) -> Self {
        Self::new(event.packet)
    }

    /// Returns the packet id.
    pub fn id(&self) -> i32 {
        self.packet.id()
    }

    /// Returns the packet direction.
    pub fn direction(&self) -> PacketDirection {
        self.packet.direction()
    }

    /// Returns the connection state.
    pub fn state(&self) -> ConnectionState {
        self.packet.state()
    }

    /// Returns `true` if this is a Java connection state.
    pub fn is_java(&self) -> bool {
        self.packet.is_java()
    }

    /// Returns `true` if this is a Bedrock connection state.
    pub fn is_bedrock(&self) -> bool {
        self.packet.is_bedrock()
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
        self.packet.java_packet_id(phase, name)
    }

    /// Resolves a Java packet `(phase, name)` by raw packet id for this player's protocol version.
    pub fn java_packet_name(&self) -> Option<java::catalog::JavaPacketName> {
        self.packet
            .java_packet_key()
            .map(|key| java::catalog::JavaPacketName {
                phase: key.phase,
                name: key.name,
            })
    }

    /// Resolves the generated typed packet key for this packet.
    pub fn java_packet_key(&self) -> Option<java::typed::JavaPacketKey> {
        let key = self.packet.java_packet_key()?;
        java::typed::JavaPacketKey::from_parts(key.direction, &key.phase, &key.name)
    }

    /// Resolves a packet id for the current version using a generated typed key.
    pub fn java_packet_id_for_key(&self, key: java::typed::JavaPacketKey) -> Option<i32> {
        self.packet.java_packet_id_for_key(&JavaPacketKey {
            direction: key.direction(),
            phase: key.phase().to_string(),
            name: key.name().to_string(),
        })
    }

    /// Returns the packet payload bytes.
    pub fn payload(&self) -> Vec<u8> {
        self.packet.payload()
    }

    /// Returns the payload length in bytes.
    pub fn payload_len(&self) -> usize {
        self.packet.payload_len() as usize
    }

    /// Creates a WIT-backed reader over the current payload.
    pub fn reader(&self) -> WitPacketReader {
        self.packet.reader()
    }

    /// Replaces the payload bytes.
    pub fn replace_payload(&self, payload: Vec<u8>) {
        self.packet.replace_payload(&payload);
    }

    /// Marks the packet event as cancelled.
    pub fn cancel(&self) {
        self.packet.cancel();
    }

    /// Returns whether the packet event is currently cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.packet.is_cancelled()
    }

    /// Sends a system-message reply to the packet sender, if the event has a player.
    ///
    /// In packet events this is executed through the host's deferred effect queue,
    /// so it is safe to call from inside raw packet handlers.
    pub fn reply_system_message(
        &self,
        text: impl Into<crate::text::TextComponent>,
        overlay: bool,
    ) -> bool {
        self.packet.reply_system_message(text.into(), overlay)
    }

    /// Sends a raw packet reply to the packet sender, if the event has a player.
    ///
    /// In packet events this is executed through the host's deferred effect queue,
    /// so it is safe to call from inside raw packet handlers.
    pub fn reply_raw_packet(&self, id: i32, payload: impl Into<Vec<u8>>) -> bool {
        self.packet.reply_raw_packet(&RawPacket::new(id, payload))
    }

    /// Sends a custom payload reply to the packet sender, if the event has a player.
    ///
    /// In packet events this is executed through the host's deferred effect queue,
    /// so it is safe to call from inside raw packet handlers.
    pub fn reply_packet_on_channel(
        &self,
        channel: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) -> bool {
        self.packet.reply_packet(&Packet::new(channel, data))
    }

    /// Decodes payload using the generated or custom-registered schema for this packet.
    pub fn decode_java_registered_schema(&self) -> Result<Vec<NamedFieldValue>, PacketSchemaError> {
        let Some(key) = self.java_packet_key() else {
            return Err(PacketSchemaError::MissingSchema);
        };

        let registry = java::schema::java_packet_schema_registry()
            .read()
            .expect("java packet schema registry poisoned");
        let Some(schema) = registry.get(key).cloned() else {
            return Err(PacketSchemaError::MissingSchema);
        };

        Ok(schema
            .decode(&self.payload())
            .map_err(to_wit_packet_schema_error)?
            .into_iter()
            .map(|(name, value)| NamedFieldValue {
                name,
                value: to_wit_field_value(value),
            })
            .collect())
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

    /// Shared map type for schema decoded fields.
    pub type SchemaFieldMap = std::collections::BTreeMap<String, schema::FieldValue>;

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

    /// Generated typed packet keys covering all Java packets in Pumpkin's supported versions.
    pub mod typed {
        include!(concat!(env!("OUT_DIR"), "/generated_java_packet_keys.rs"));
    }

    /// Field-level schema helpers for packet parsing/encoding.
    pub mod schema {
        use std::{
            collections::BTreeMap,
            sync::{OnceLock, RwLock},
        };

        use crate::packet::{BlockPos, Vec3F32, Vec3F64, Vec3I16, Vec3I32};

        use super::{PacketReadError, PacketReader, PacketWriter, typed::JavaPacketKey};

        /// Supported primitive field types for schema-based parsing.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum FieldType {
            U8,
            Bool,
            U16,
            I8,
            I16,
            I32,
            I64,
            F32,
            F64,
            BlockPos,
            Vec3F32,
            Vec3F64,
            Vec3I16,
            Vec3I32,
            VarInt,
            VarLong,
            String { max_len: usize },
            Bytes { len: usize },
            RemainingBytes,
            UuidBytes,
            Optional(Box<FieldType>),
        }

        /// Parsed value for a schema field.
        #[derive(Debug, Clone)]
        pub enum FieldValue {
            U8(u8),
            Bool(bool),
            U16(u16),
            I8(i8),
            I16(i16),
            I32(i32),
            I64(i64),
            F32(u32),
            F64(u64),
            BlockPos(BlockPos),
            Vec3F32(Vec3F32),
            Vec3F64(Vec3F64),
            Vec3I16(Vec3I16),
            Vec3I32(Vec3I32),
            VarInt(i32),
            VarLong(i64),
            String(String),
            Bytes(Vec<u8>),
            UuidBytes([u8; 16]),
        }

        impl PartialEq for FieldValue {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::U8(a), Self::U8(b)) => a == b,
                    (Self::Bool(a), Self::Bool(b)) => a == b,
                    (Self::U16(a), Self::U16(b)) => a == b,
                    (Self::I8(a), Self::I8(b)) => a == b,
                    (Self::I16(a), Self::I16(b)) => a == b,
                    (Self::I32(a), Self::I32(b)) => a == b,
                    (Self::I64(a), Self::I64(b)) => a == b,
                    (Self::F32(a), Self::F32(b)) => a == b,
                    (Self::F64(a), Self::F64(b)) => a == b,
                    (Self::BlockPos(a), Self::BlockPos(b)) => {
                        a.x == b.x && a.y == b.y && a.z == b.z
                    }
                    (Self::Vec3F32(a), Self::Vec3F32(b)) => {
                        a.x.to_bits() == b.x.to_bits()
                            && a.y.to_bits() == b.y.to_bits()
                            && a.z.to_bits() == b.z.to_bits()
                    }
                    (Self::Vec3F64(a), Self::Vec3F64(b)) => {
                        a.x.to_bits() == b.x.to_bits()
                            && a.y.to_bits() == b.y.to_bits()
                            && a.z.to_bits() == b.z.to_bits()
                    }
                    (Self::Vec3I16(a), Self::Vec3I16(b)) => a.x == b.x && a.y == b.y && a.z == b.z,
                    (Self::Vec3I32(a), Self::Vec3I32(b)) => a.x == b.x && a.y == b.y && a.z == b.z,
                    (Self::VarInt(a), Self::VarInt(b)) => a == b,
                    (Self::VarLong(a), Self::VarLong(b)) => a == b,
                    (Self::String(a), Self::String(b)) => a == b,
                    (Self::Bytes(a), Self::Bytes(b)) => a == b,
                    (Self::UuidBytes(a), Self::UuidBytes(b)) => a == b,
                    _ => false,
                }
            }
        }

        /// A single schema field definition.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Field {
            pub name: String,
            pub kind: FieldType,
        }

        /// Schema for a single packet payload.
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        pub struct PacketSchema {
            fields: Vec<Field>,
        }

        /// Errors for schema decode/encode operations.
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum PacketSchemaError {
            Decode(PacketReadError),
            MissingField(String),
            WrongType { field: String },
        }

        impl PacketSchema {
            /// Creates an empty schema.
            pub fn new() -> Self {
                Self::default()
            }

            /// Appends a field to the schema.
            pub fn field(mut self, name: impl Into<String>, kind: FieldType) -> Self {
                self.fields.push(Field {
                    name: name.into(),
                    kind,
                });
                self
            }

            /// Appends an optional field to the schema.
            pub fn field_optional(mut self, name: impl Into<String>, kind: FieldType) -> Self {
                self.fields.push(Field {
                    name: name.into(),
                    kind: FieldType::Optional(Box::new(kind)),
                });
                self
            }

            /// Returns immutable schema fields.
            pub fn fields(&self) -> &[Field] {
                &self.fields
            }

            /// Decodes payload bytes according to this schema.
            pub fn decode(
                &self,
                payload: &[u8],
            ) -> Result<BTreeMap<String, FieldValue>, PacketSchemaError> {
                let mut reader = PacketReader::new(payload);
                let mut out = BTreeMap::new();

                for field in &self.fields {
                    if let Some(value) = decode_field_value(&field.kind, &mut reader)
                        .map_err(PacketSchemaError::Decode)?
                    {
                        out.insert(field.name.clone(), value);
                    }
                }

                Ok(out)
            }

            /// Encodes values according to this schema.
            pub fn encode(
                &self,
                values: &BTreeMap<String, FieldValue>,
            ) -> Result<Vec<u8>, PacketSchemaError> {
                let mut writer = PacketWriter::new();

                for field in &self.fields {
                    encode_field_value(&field.name, &field.kind, values, &mut writer)?;
                }

                Ok(writer.into_inner())
            }
        }

        fn decode_field_value(
            kind: &FieldType,
            reader: &mut PacketReader<'_>,
        ) -> Result<Option<FieldValue>, PacketReadError> {
            let value = match kind {
                FieldType::U8 => Some(FieldValue::U8(reader.read_u8()?)),
                FieldType::Bool => Some(FieldValue::Bool(reader.read_bool()?)),
                FieldType::U16 => Some(FieldValue::U16(reader.read_u16()?)),
                FieldType::I8 => Some(FieldValue::I8(reader.read_i8()?)),
                FieldType::I16 => Some(FieldValue::I16(reader.read_i16()?)),
                FieldType::I32 => Some(FieldValue::I32(reader.read_i32()?)),
                FieldType::I64 => Some(FieldValue::I64(reader.read_i64()?)),
                FieldType::F32 => Some(FieldValue::F32(reader.read_f32()?.to_bits())),
                FieldType::F64 => Some(FieldValue::F64(reader.read_f64()?.to_bits())),
                FieldType::BlockPos => Some(FieldValue::BlockPos(reader.read_block_pos()?)),
                FieldType::Vec3F32 => Some(FieldValue::Vec3F32(reader.read_vec3_f32()?)),
                FieldType::Vec3F64 => Some(FieldValue::Vec3F64(reader.read_vec3_f64()?)),
                FieldType::Vec3I16 => Some(FieldValue::Vec3I16(reader.read_vec3_i16()?)),
                FieldType::Vec3I32 => Some(FieldValue::Vec3I32(reader.read_vec3_i32()?)),
                FieldType::VarInt => Some(FieldValue::VarInt(reader.read_varint()?)),
                FieldType::VarLong => Some(FieldValue::VarLong(reader.read_varlong()?)),
                FieldType::String { max_len } => {
                    Some(FieldValue::String(reader.read_string(*max_len)?))
                }
                FieldType::Bytes { len } => {
                    Some(FieldValue::Bytes(reader.read_bytes(*len)?.to_vec()))
                }
                FieldType::RemainingBytes => Some(FieldValue::Bytes(
                    reader.read_bytes(reader.remaining())?.to_vec(),
                )),
                FieldType::UuidBytes => Some(FieldValue::UuidBytes(reader.read_uuid_bytes()?)),
                FieldType::Optional(inner) => {
                    if !reader.read_bool()? {
                        None
                    } else {
                        decode_field_value(inner, reader)?
                    }
                }
            };

            Ok(value)
        }

        fn encode_field_value(
            field_name: &str,
            kind: &FieldType,
            values: &BTreeMap<String, FieldValue>,
            writer: &mut PacketWriter,
        ) -> Result<(), PacketSchemaError> {
            match kind {
                FieldType::Optional(inner) => {
                    if let Some(value) = values.get(field_name) {
                        writer.write_bool(true);
                        encode_value(field_name, inner, value, writer)
                    } else {
                        writer.write_bool(false);
                        Ok(())
                    }
                }
                _ => {
                    let value = values
                        .get(field_name)
                        .ok_or_else(|| PacketSchemaError::MissingField(field_name.to_string()))?;
                    encode_value(field_name, kind, value, writer)
                }
            }
        }

        fn encode_value(
            field_name: &str,
            kind: &FieldType,
            value: &FieldValue,
            writer: &mut PacketWriter,
        ) -> Result<(), PacketSchemaError> {
            match (kind, value) {
                (FieldType::U8, FieldValue::U8(v)) => writer.write_u8(*v),
                (FieldType::Bool, FieldValue::Bool(v)) => writer.write_bool(*v),
                (FieldType::U16, FieldValue::U16(v)) => writer.write_u16(*v),
                (FieldType::I8, FieldValue::I8(v)) => writer.write_i8(*v),
                (FieldType::I16, FieldValue::I16(v)) => writer.write_i16(*v),
                (FieldType::I32, FieldValue::I32(v)) => writer.write_i32(*v),
                (FieldType::I64, FieldValue::I64(v)) => writer.write_i64(*v),
                (FieldType::F32, FieldValue::F32(v)) => writer.write_f32(f32::from_bits(*v)),
                (FieldType::F64, FieldValue::F64(v)) => writer.write_f64(f64::from_bits(*v)),
                (FieldType::BlockPos, FieldValue::BlockPos(v)) => writer.write_block_pos(v),
                (FieldType::Vec3F32, FieldValue::Vec3F32(v)) => writer.write_vec3_f32(v),
                (FieldType::Vec3F64, FieldValue::Vec3F64(v)) => writer.write_vec3_f64(v),
                (FieldType::Vec3I16, FieldValue::Vec3I16(v)) => writer.write_vec3_i16(v),
                (FieldType::Vec3I32, FieldValue::Vec3I32(v)) => writer.write_vec3_i32(v),
                (FieldType::VarInt, FieldValue::VarInt(v)) => writer.write_varint(*v),
                (FieldType::VarLong, FieldValue::VarLong(v)) => writer.write_varlong(*v),
                (FieldType::String { .. }, FieldValue::String(v)) => writer.write_string(v),
                (FieldType::Bytes { .. } | FieldType::RemainingBytes, FieldValue::Bytes(v)) => {
                    writer.write_bytes(v)
                }
                (FieldType::UuidBytes, FieldValue::UuidBytes(v)) => writer.write_uuid_bytes(v),
                _ => {
                    return Err(PacketSchemaError::WrongType {
                        field: field_name.to_string(),
                    });
                }
            }

            Ok(())
        }

        /// Global registry for packet schemas keyed by generated packet key.
        #[derive(Debug, Default)]
        pub struct JavaPacketSchemaRegistry {
            inner: BTreeMap<JavaPacketKey, PacketSchema>,
        }

        impl JavaPacketSchemaRegistry {
            /// Registers or replaces a schema for a key.
            pub fn register(&mut self, key: JavaPacketKey, schema: PacketSchema) {
                self.inner.insert(key, schema);
            }

            /// Gets a schema for a key.
            pub fn get(&self, key: JavaPacketKey) -> Option<&PacketSchema> {
                self.inner.get(&key)
            }
        }

        include!(concat!(
            env!("OUT_DIR"),
            "/generated_java_packet_schemas.rs"
        ));

        static GLOBAL_SCHEMA_REGISTRY: OnceLock<RwLock<JavaPacketSchemaRegistry>> = OnceLock::new();

        /// Returns the global schema registry.
        pub fn java_packet_schema_registry() -> &'static RwLock<JavaPacketSchemaRegistry> {
            GLOBAL_SCHEMA_REGISTRY
                .get_or_init(|| RwLock::new(generated_java_packet_schema_registry()))
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
