use bytes::Bytes;
use pumpkin_util::version::MinecraftVersion;
use serde::Deserialize;
use std::{collections::BTreeMap, sync::OnceLock};
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{PacketHandleResource, PacketReaderResource, PacketWriterResource, PluginHostState},
    wit::v0_1::pumpkin::{
        self,
        plugin::{
            packet::{
                BedrockState, BlockPos, ConnectionState, FieldValue, JavaPacketKey, JavaState,
                NamedFieldValue, Packet, PacketDirection, PacketHandle, PacketReadError,
                PacketReader, PacketSchemaError, PacketWriter, RawPacket, Vec3F32, Vec3F64,
                Vec3I16, Vec3I32,
            },
            text::TextComponent,
        },
    },
};

#[derive(Clone)]
pub struct PluginPacketHandle {
    pub event: crate::plugin::packet::RawPacketEvent,
}

pub struct PluginPacketReader {
    pub buf: Vec<u8>,
    pub pos: usize,
}

#[derive(Default)]
pub struct PluginPacketWriter {
    pub buf: Vec<u8>,
}

fn packet_handle_from_resource(
    state: &PluginHostState,
    packet: &Resource<PacketHandle>,
) -> wasmtime::Result<PluginPacketHandle> {
    state
        .resource_table
        .get::<PacketHandleResource>(&Resource::new_own(packet.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid packet-handle resource"))
        .map(|resource| resource.provider.clone())
}

fn packet_handle_from_resource_mut<'a>(
    state: &'a mut PluginHostState,
    packet: &Resource<PacketHandle>,
) -> wasmtime::Result<&'a mut PluginPacketHandle> {
    state
        .resource_table
        .get_mut::<PacketHandleResource>(&Resource::new_own(packet.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid packet-handle resource"))
        .map(|resource| &mut resource.provider)
}

fn packet_reader_from_resource_mut<'a>(
    state: &'a mut PluginHostState,
    reader: &Resource<PacketReader>,
) -> wasmtime::Result<&'a mut PluginPacketReader> {
    state
        .resource_table
        .get_mut::<PacketReaderResource>(&Resource::new_own(reader.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid packet-reader resource"))
        .map(|resource| &mut resource.provider)
}

fn packet_writer_from_resource_mut<'a>(
    state: &'a mut PluginHostState,
    writer: &Resource<PacketWriter>,
) -> wasmtime::Result<&'a mut PluginPacketWriter> {
    state
        .resource_table
        .get_mut::<PacketWriterResource>(&Resource::new_own(writer.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid packet-writer resource"))
        .map(|resource| &mut resource.provider)
}

fn packet_writer_from_resource<'a>(
    state: &'a PluginHostState,
    writer: &Resource<PacketWriter>,
) -> wasmtime::Result<&'a PluginPacketWriter> {
    state
        .resource_table
        .get::<PacketWriterResource>(&Resource::new_own(writer.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid packet-writer resource"))
        .map(|resource| &resource.provider)
}

const fn to_wasm_direction(direction: crate::plugin::packet::PacketDirection) -> PacketDirection {
    match direction {
        crate::plugin::packet::PacketDirection::Serverbound => PacketDirection::Serverbound,
        crate::plugin::packet::PacketDirection::Clientbound => PacketDirection::Clientbound,
    }
}

const fn to_wasm_state(state: crate::plugin::packet::PacketConnectionState) -> ConnectionState {
    match state {
        crate::plugin::packet::PacketConnectionState::Java(state) => {
            ConnectionState::Java(match state {
                crate::plugin::packet::JavaConnectionState::Handshake => JavaState::Handshake,
                crate::plugin::packet::JavaConnectionState::Status => JavaState::Status,
                crate::plugin::packet::JavaConnectionState::Login => JavaState::Login,
                crate::plugin::packet::JavaConnectionState::Config => JavaState::Config,
                crate::plugin::packet::JavaConnectionState::Play => JavaState::Play,
                crate::plugin::packet::JavaConnectionState::Transfer => JavaState::Transfer,
            })
        }
        crate::plugin::packet::PacketConnectionState::Bedrock(state) => {
            ConnectionState::Bedrock(match state {
                crate::plugin::packet::BedrockConnectionState::Offline => BedrockState::Offline,
                crate::plugin::packet::BedrockConnectionState::Raknet => BedrockState::Raknet,
                crate::plugin::packet::BedrockConnectionState::Game => BedrockState::Game,
            })
        }
    }
}

const fn to_wasm_key(
    key: crate::plugin::packet::PacketDirection,
    phase: String,
    name: String,
) -> JavaPacketKey {
    JavaPacketKey {
        direction: to_wasm_direction(key),
        phase,
        name,
    }
}

const fn protocol_to_mc_version(protocol: u32) -> MinecraftVersion {
    MinecraftVersion::from_protocol(protocol)
}

#[derive(Debug, Deserialize)]
struct VersionPacketAsset {
    #[allow(dead_code)]
    version: u32,
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
}

#[derive(Debug, Clone)]
struct JavaPacketNameLocal {
    phase: String,
    name: String,
}

#[derive(Debug, Default)]
struct VersionPacketsLocal {
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
    reverse_serverbound: BTreeMap<i32, JavaPacketNameLocal>,
    reverse_clientbound: BTreeMap<i32, JavaPacketNameLocal>,
}

#[derive(Debug, Default)]
struct JavaPacketCatalogLocal {
    versions: BTreeMap<MinecraftVersion, VersionPacketsLocal>,
}

const EMBEDDED_PACKET_ASSETS: &[(MinecraftVersion, &str)] = &[
    (
        MinecraftVersion::V_1_20_5,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../assets/packet/1_20_5_packets.json"
        )),
    ),
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
];

fn build_reverse_lookup(
    packets: &BTreeMap<String, BTreeMap<String, i32>>,
) -> BTreeMap<i32, JavaPacketNameLocal> {
    let mut reverse = BTreeMap::new();

    for (phase, phase_packets) in packets {
        for (name, id) in phase_packets {
            reverse.insert(
                *id,
                JavaPacketNameLocal {
                    phase: phase.clone(),
                    name: name.clone(),
                },
            );
        }
    }

    reverse
}

fn parse_version_packets(json: &str) -> VersionPacketsLocal {
    let parsed: VersionPacketAsset =
        serde_json::from_str(json).expect("failed to parse embedded packet asset");

    VersionPacketsLocal {
        reverse_serverbound: build_reverse_lookup(&parsed.serverbound),
        reverse_clientbound: build_reverse_lookup(&parsed.clientbound),
        serverbound: parsed.serverbound,
        clientbound: parsed.clientbound,
    }
}

impl JavaPacketCatalogLocal {
    fn load_embedded() -> Self {
        let mut catalog = Self::default();

        for &(version, json) in EMBEDDED_PACKET_ASSETS {
            catalog
                .versions
                .insert(version, parse_version_packets(json));
        }

        catalog
    }

    fn get_id(
        &self,
        version: MinecraftVersion,
        direction: crate::plugin::packet::PacketDirection,
        phase: &str,
        name: &str,
    ) -> Option<i32> {
        let packets = self.versions.get(&version)?;
        let phase_packets = match direction {
            crate::plugin::packet::PacketDirection::Serverbound => {
                packets.serverbound.get(phase)?
            }
            crate::plugin::packet::PacketDirection::Clientbound => {
                packets.clientbound.get(phase)?
            }
        };
        phase_packets.get(name).copied()
    }

    fn get_name(
        &self,
        version: MinecraftVersion,
        direction: crate::plugin::packet::PacketDirection,
        id: i32,
    ) -> Option<JavaPacketNameLocal> {
        let packets = self.versions.get(&version)?;
        match direction {
            crate::plugin::packet::PacketDirection::Serverbound => {
                packets.reverse_serverbound.get(&id).cloned()
            }
            crate::plugin::packet::PacketDirection::Clientbound => {
                packets.reverse_clientbound.get(&id).cloned()
            }
        }
    }
}

fn java_packet_catalog() -> &'static JavaPacketCatalogLocal {
    static JAVA_PACKET_CATALOG: OnceLock<JavaPacketCatalogLocal> = OnceLock::new();
    JAVA_PACKET_CATALOG.get_or_init(JavaPacketCatalogLocal::load_embedded)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum LocalFieldType {
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
    Optional(Box<Self>),
}

#[derive(Debug, Clone)]
struct LocalField {
    name: String,
    kind: LocalFieldType,
}

#[derive(Debug, Clone, Default)]
struct LocalPacketSchema {
    fields: Vec<LocalField>,
}

impl LocalPacketSchema {
    fn new() -> Self {
        Self::default()
    }

    fn field(mut self, name: impl Into<String>, kind: LocalFieldType) -> Self {
        self.fields.push(LocalField {
            name: name.into(),
            kind,
        });
        self
    }

    fn decode(&self, payload: &[u8]) -> Result<Vec<NamedFieldValue>, PacketSchemaError> {
        let mut reader = PluginPacketReader {
            buf: payload.to_vec(),
            pos: 0,
        };
        let mut out = Vec::new();

        for field in &self.fields {
            if let Some(value) = decode_local_field_value(&field.kind, &mut reader)? {
                out.push(NamedFieldValue {
                    name: field.name.clone(),
                    value,
                });
            }
        }

        Ok(out)
    }
}

fn schema_key(direction: PacketDirection, phase: &str, name: &str) -> String {
    let direction = match direction {
        PacketDirection::Serverbound => "serverbound",
        PacketDirection::Clientbound => "clientbound",
    };
    format!("{direction}:{phase}:{name}")
}

include!(concat!(
    env!("OUT_DIR"),
    "/generated_java_packet_schemas.rs"
));

fn java_packet_schema_registry() -> &'static BTreeMap<String, LocalPacketSchema> {
    static JAVA_PACKET_SCHEMA_REGISTRY: OnceLock<BTreeMap<String, LocalPacketSchema>> =
        OnceLock::new();
    JAVA_PACKET_SCHEMA_REGISTRY.get_or_init(generated_java_packet_schema_registry)
}

fn decode_local_field_value(
    kind: &LocalFieldType,
    reader: &mut PluginPacketReader,
) -> Result<Option<FieldValue>, PacketSchemaError> {
    let value = match kind {
        LocalFieldType::U8 => Some(FieldValue::U8Value(
            reader.read_u8().map_err(to_schema_error)?,
        )),
        LocalFieldType::Bool => Some(FieldValue::BoolValue(
            reader.read_bool().map_err(to_schema_error)?,
        )),
        LocalFieldType::U16 => Some(FieldValue::U16Value(
            reader.read_u16().map_err(to_schema_error)?,
        )),
        LocalFieldType::I8 => Some(FieldValue::I8Value(
            reader.read_i8().map_err(to_schema_error)?,
        )),
        LocalFieldType::I16 => Some(FieldValue::I16Value(
            reader.read_i16().map_err(to_schema_error)?,
        )),
        LocalFieldType::I32 => Some(FieldValue::I32Value(
            reader.read_i32().map_err(to_schema_error)?,
        )),
        LocalFieldType::I64 => Some(FieldValue::I64Value(
            reader.read_i64().map_err(to_schema_error)?,
        )),
        LocalFieldType::F32 => Some(FieldValue::F32Value(
            reader.read_f32().map_err(to_schema_error)?,
        )),
        LocalFieldType::F64 => Some(FieldValue::F64Value(
            reader.read_f64().map_err(to_schema_error)?,
        )),
        LocalFieldType::BlockPos => Some(FieldValue::BlockPosValue(
            reader.read_block_pos().map_err(to_schema_error)?,
        )),
        LocalFieldType::Vec3F32 => Some(FieldValue::Vec3F32Value(
            reader.read_vec3_f32().map_err(to_schema_error)?,
        )),
        LocalFieldType::Vec3F64 => Some(FieldValue::Vec3F64Value(
            reader.read_vec3_f64().map_err(to_schema_error)?,
        )),
        LocalFieldType::Vec3I16 => Some(FieldValue::Vec3I16Value(
            reader.read_vec3_i16().map_err(to_schema_error)?,
        )),
        LocalFieldType::Vec3I32 => Some(FieldValue::Vec3I32Value(
            reader.read_vec3_i32().map_err(to_schema_error)?,
        )),
        LocalFieldType::VarInt => Some(FieldValue::VarintValue(
            reader.read_varint().map_err(to_schema_error)?,
        )),
        LocalFieldType::VarLong => Some(FieldValue::VarlongValue(
            reader.read_varlong().map_err(to_schema_error)?,
        )),
        LocalFieldType::String { max_len } => Some(FieldValue::StringValue(
            reader.read_string(*max_len).map_err(to_schema_error)?,
        )),
        LocalFieldType::Bytes { len } => Some(FieldValue::BytesValue(
            reader.read_exact(*len).map_err(to_schema_error)?.to_vec(),
        )),
        LocalFieldType::RemainingBytes => Some(FieldValue::BytesValue(
            reader
                .read_exact(reader.remaining() as usize)
                .map_err(to_schema_error)?
                .to_vec(),
        )),
        LocalFieldType::UuidBytes => Some(FieldValue::UuidBytesValue(
            reader.read_exact(16).map_err(to_schema_error)?.to_vec(),
        )),
        LocalFieldType::Optional(inner) => {
            if reader.read_bool().map_err(to_schema_error)? {
                decode_local_field_value(inner, reader)?
            } else {
                None
            }
        }
    };

    Ok(value)
}

const fn to_packet_read_error(err: PacketReadErrorLocal) -> PacketReadError {
    match err {
        PacketReadErrorLocal::UnexpectedEof => PacketReadError::UnexpectedEof,
        PacketReadErrorLocal::VarIntTooLong => PacketReadError::VarintTooLong,
        PacketReadErrorLocal::StringTooLong => PacketReadError::StringTooLong,
        PacketReadErrorLocal::InvalidUtf8 => PacketReadError::InvalidUtf8,
    }
}

const fn to_schema_error(err: PacketReadErrorLocal) -> PacketSchemaError {
    match err {
        PacketReadErrorLocal::UnexpectedEof => PacketSchemaError::DecodeUnexpectedEof,
        PacketReadErrorLocal::VarIntTooLong => PacketSchemaError::DecodeVarintTooLong,
        PacketReadErrorLocal::StringTooLong => PacketSchemaError::DecodeStringTooLong,
        PacketReadErrorLocal::InvalidUtf8 => PacketSchemaError::DecodeInvalidUtf8,
    }
}

#[derive(Clone, Copy)]
enum PacketReadErrorLocal {
    UnexpectedEof,
    VarIntTooLong,
    StringTooLong,
    InvalidUtf8,
}

impl PluginPacketReader {
    const fn remaining(&self) -> u32 {
        self.buf.len().saturating_sub(self.pos) as u32
    }

    fn read_exact(&mut self, len: usize) -> Result<&[u8], PacketReadErrorLocal> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or(PacketReadErrorLocal::UnexpectedEof)?;
        let slice = self
            .buf
            .get(self.pos..end)
            .ok_or(PacketReadErrorLocal::UnexpectedEof)?;
        self.pos = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, PacketReadErrorLocal> {
        Ok(*self.read_exact(1)?.first().unwrap())
    }

    fn read_bool(&mut self) -> Result<bool, PacketReadErrorLocal> {
        Ok(self.read_u8()? != 0)
    }

    fn read_u16(&mut self) -> Result<u16, PacketReadErrorLocal> {
        let bytes = self.read_exact(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_i8(&mut self) -> Result<i8, PacketReadErrorLocal> {
        Ok(self.read_u8()? as i8)
    }

    fn read_i16(&mut self) -> Result<i16, PacketReadErrorLocal> {
        let bytes = self.read_exact(2)?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_i32(&mut self) -> Result<i32, PacketReadErrorLocal> {
        let bytes = self.read_exact(4)?;
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i64(&mut self) -> Result<i64, PacketReadErrorLocal> {
        let bytes = self.read_exact(8)?;
        Ok(i64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_f32(&mut self) -> Result<f32, PacketReadErrorLocal> {
        let bytes = self.read_exact(4)?;
        Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_f64(&mut self) -> Result<f64, PacketReadErrorLocal> {
        let bytes = self.read_exact(8)?;
        Ok(f64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_block_pos(&mut self) -> Result<BlockPos, PacketReadErrorLocal> {
        let encoded = self.read_i64()?;
        Ok(BlockPos {
            x: (encoded >> 38) as i32,
            y: (encoded << 52 >> 52) as i32,
            z: (encoded << 26 >> 38) as i32,
        })
    }

    fn read_vec3_f32(&mut self) -> Result<Vec3F32, PacketReadErrorLocal> {
        Ok(Vec3F32 {
            x: self.read_f32()?,
            y: self.read_f32()?,
            z: self.read_f32()?,
        })
    }

    fn read_vec3_f64(&mut self) -> Result<Vec3F64, PacketReadErrorLocal> {
        Ok(Vec3F64 {
            x: self.read_f64()?,
            y: self.read_f64()?,
            z: self.read_f64()?,
        })
    }

    fn read_vec3_i16(&mut self) -> Result<Vec3I16, PacketReadErrorLocal> {
        Ok(Vec3I16 {
            x: self.read_i16()?,
            y: self.read_i16()?,
            z: self.read_i16()?,
        })
    }

    fn read_vec3_i32(&mut self) -> Result<Vec3I32, PacketReadErrorLocal> {
        Ok(Vec3I32 {
            x: self.read_i32()?,
            y: self.read_i32()?,
            z: self.read_i32()?,
        })
    }

    fn read_varint(&mut self) -> Result<i32, PacketReadErrorLocal> {
        let mut num_read = 0u32;
        let mut result = 0i32;

        loop {
            let byte = self.read_u8()?;
            let value = i32::from(byte & 0x7F);
            result |= value << (7 * num_read);
            num_read += 1;
            if num_read > 5 {
                return Err(PacketReadErrorLocal::VarIntTooLong);
            }
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
        }
    }

    fn read_varlong(&mut self) -> Result<i64, PacketReadErrorLocal> {
        let mut num_read = 0u32;
        let mut result = 0i64;

        loop {
            let byte = self.read_u8()?;
            let value = i64::from(byte & 0x7F);
            result |= value << (7 * num_read);
            num_read += 1;
            if num_read > 10 {
                return Err(PacketReadErrorLocal::VarIntTooLong);
            }
            if (byte & 0x80) == 0 {
                return Ok(result);
            }
        }
    }

    fn read_string(&mut self, max_len: usize) -> Result<String, PacketReadErrorLocal> {
        let len = usize::try_from(self.read_varint()?)
            .map_err(|_| PacketReadErrorLocal::StringTooLong)?;
        if len > max_len {
            return Err(PacketReadErrorLocal::StringTooLong);
        }
        let bytes = self.read_exact(len)?;
        core::str::from_utf8(bytes)
            .map(std::string::ToString::to_string)
            .map_err(|_| PacketReadErrorLocal::InvalidUtf8)
    }
}

impl pumpkin::plugin::packet::Host for PluginHostState {
    async fn new_packet_writer(&mut self) -> wasmtime::Result<Resource<PacketWriter>> {
        self.add_packet_writer(PluginPacketWriter::default())
    }
}

impl pumpkin::plugin::packet::HostPacketReader for PluginHostState {
    async fn remaining(&mut self, reader: Resource<PacketReader>) -> wasmtime::Result<u32> {
        Ok(packet_reader_from_resource_mut(self, &reader)?.remaining())
    }

    async fn position(&mut self, reader: Resource<PacketReader>) -> wasmtime::Result<u32> {
        Ok(packet_reader_from_resource_mut(self, &reader)?.pos as u32)
    }

    async fn read_u8(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<u8, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_u8()
            .map_err(to_packet_read_error))
    }

    async fn read_bool(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<bool, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_bool()
            .map_err(to_packet_read_error))
    }

    async fn read_u16(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<u16, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_u16()
            .map_err(to_packet_read_error))
    }

    async fn read_i8(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<i8, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_i8()
            .map_err(to_packet_read_error))
    }

    async fn read_i16(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<i16, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_i16()
            .map_err(to_packet_read_error))
    }

    async fn read_i32(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<i32, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_i32()
            .map_err(to_packet_read_error))
    }

    async fn read_i64(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<i64, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_i64()
            .map_err(to_packet_read_error))
    }

    async fn read_f32(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<f32, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_f32()
            .map_err(to_packet_read_error))
    }

    async fn read_f64(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<f64, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_f64()
            .map_err(to_packet_read_error))
    }

    async fn read_block_pos(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<BlockPos, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_block_pos()
            .map_err(to_packet_read_error))
    }

    async fn read_vec3_f32(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<Vec3F32, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_vec3_f32()
            .map_err(to_packet_read_error))
    }

    async fn read_vec3_f64(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<Vec3F64, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_vec3_f64()
            .map_err(to_packet_read_error))
    }

    async fn read_vec3_i16(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<Vec3I16, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_vec3_i16()
            .map_err(to_packet_read_error))
    }

    async fn read_vec3_i32(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<Vec3I32, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_vec3_i32()
            .map_err(to_packet_read_error))
    }

    async fn read_varint(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<i32, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_varint()
            .map_err(to_packet_read_error))
    }

    async fn read_varlong(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<i64, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_varlong()
            .map_err(to_packet_read_error))
    }

    async fn read_bytes(
        &mut self,
        reader: Resource<PacketReader>,
        len: u32,
    ) -> wasmtime::Result<Result<Vec<u8>, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_exact(len as usize)
            .map(<[u8]>::to_vec)
            .map_err(to_packet_read_error))
    }

    async fn read_uuid_bytes(
        &mut self,
        reader: Resource<PacketReader>,
    ) -> wasmtime::Result<Result<Vec<u8>, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_exact(16)
            .map(<[u8]>::to_vec)
            .map_err(to_packet_read_error))
    }

    async fn read_string(
        &mut self,
        reader: Resource<PacketReader>,
        max_len: u32,
    ) -> wasmtime::Result<Result<String, PacketReadError>> {
        Ok(packet_reader_from_resource_mut(self, &reader)?
            .read_string(max_len as usize)
            .map_err(to_packet_read_error))
    }

    async fn drop(&mut self, reader: Resource<PacketReader>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<PacketReaderResource>(Resource::new_own(reader.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::packet::HostPacketWriter for PluginHostState {
    async fn write_u8(
        &mut self,
        writer: Resource<PacketWriter>,
        value: u8,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .push(value);
        Ok(())
    }

    async fn write_bool(
        &mut self,
        writer: Resource<PacketWriter>,
        value: bool,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .push(u8::from(value));
        Ok(())
    }

    async fn write_u16(
        &mut self,
        writer: Resource<PacketWriter>,
        value: u16,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    async fn write_i8(
        &mut self,
        writer: Resource<PacketWriter>,
        value: i8,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .push(value as u8);
        Ok(())
    }

    async fn write_i16(
        &mut self,
        writer: Resource<PacketWriter>,
        value: i16,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    async fn write_i32(
        &mut self,
        writer: Resource<PacketWriter>,
        value: i32,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    async fn write_i64(
        &mut self,
        writer: Resource<PacketWriter>,
        value: i64,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    async fn write_f32(
        &mut self,
        writer: Resource<PacketWriter>,
        value: f32,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    async fn write_f64(
        &mut self,
        writer: Resource<PacketWriter>,
        value: f64,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    async fn write_block_pos(
        &mut self,
        writer: Resource<PacketWriter>,
        value: BlockPos,
    ) -> wasmtime::Result<()> {
        let encoded = ((i64::from(value.x) & 0x3FFFFFF) << 38)
            | ((i64::from(value.z) & 0x3FFFFFF) << 12)
            | (i64::from(value.y) & 0xFFF);
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&encoded.to_be_bytes());
        Ok(())
    }

    async fn write_vec3_f32(
        &mut self,
        writer: Resource<PacketWriter>,
        value: Vec3F32,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        writer.buf.extend_from_slice(&value.x.to_be_bytes());
        writer.buf.extend_from_slice(&value.y.to_be_bytes());
        writer.buf.extend_from_slice(&value.z.to_be_bytes());
        Ok(())
    }

    async fn write_vec3_f64(
        &mut self,
        writer: Resource<PacketWriter>,
        value: Vec3F64,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        writer.buf.extend_from_slice(&value.x.to_be_bytes());
        writer.buf.extend_from_slice(&value.y.to_be_bytes());
        writer.buf.extend_from_slice(&value.z.to_be_bytes());
        Ok(())
    }

    async fn write_vec3_i16(
        &mut self,
        writer: Resource<PacketWriter>,
        value: Vec3I16,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        writer.buf.extend_from_slice(&value.x.to_be_bytes());
        writer.buf.extend_from_slice(&value.y.to_be_bytes());
        writer.buf.extend_from_slice(&value.z.to_be_bytes());
        Ok(())
    }

    async fn write_vec3_i32(
        &mut self,
        writer: Resource<PacketWriter>,
        value: Vec3I32,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        writer.buf.extend_from_slice(&value.x.to_be_bytes());
        writer.buf.extend_from_slice(&value.y.to_be_bytes());
        writer.buf.extend_from_slice(&value.z.to_be_bytes());
        Ok(())
    }

    async fn write_varint(
        &mut self,
        writer: Resource<PacketWriter>,
        mut value: i32,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        loop {
            let mut temp = (value & 0x7F) as u8;
            value = ((value as u32) >> 7) as i32;
            if value != 0 {
                temp |= 0x80;
            }
            writer.buf.push(temp);
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    async fn write_varlong(
        &mut self,
        writer: Resource<PacketWriter>,
        mut value: i64,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        loop {
            let mut temp = (value & 0x7F) as u8;
            value = ((value as u64) >> 7) as i64;
            if value != 0 {
                temp |= 0x80;
            }
            writer.buf.push(temp);
            if value == 0 {
                break;
            }
        }
        Ok(())
    }

    async fn write_bytes(
        &mut self,
        writer: Resource<PacketWriter>,
        value: Vec<u8>,
    ) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value);
        Ok(())
    }

    async fn write_uuid_bytes(
        &mut self,
        writer: Resource<PacketWriter>,
        value: Vec<u8>,
    ) -> wasmtime::Result<()> {
        if value.len() != 16 {
            return Err(wasmtime::Error::msg(
                "uuid-bytes must contain exactly 16 bytes",
            ));
        }
        packet_writer_from_resource_mut(self, &writer)?
            .buf
            .extend_from_slice(&value);
        Ok(())
    }

    async fn write_string(
        &mut self,
        writer: Resource<PacketWriter>,
        value: String,
    ) -> wasmtime::Result<()> {
        let writer = packet_writer_from_resource_mut(self, &writer)?;
        let mut len = value.len() as i32;
        loop {
            let mut temp = (len & 0x7F) as u8;
            len = ((len as u32) >> 7) as i32;
            if len != 0 {
                temp |= 0x80;
            }
            writer.buf.push(temp);
            if len == 0 {
                break;
            }
        }
        writer.buf.extend_from_slice(value.as_bytes());
        Ok(())
    }

    async fn bytes(&mut self, writer: Resource<PacketWriter>) -> wasmtime::Result<Vec<u8>> {
        Ok(packet_writer_from_resource(self, &writer)?.buf.clone())
    }

    async fn reset(&mut self, writer: Resource<PacketWriter>) -> wasmtime::Result<()> {
        packet_writer_from_resource_mut(self, &writer)?.buf.clear();
        Ok(())
    }

    async fn drop(&mut self, writer: Resource<PacketWriter>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<PacketWriterResource>(Resource::new_own(writer.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::packet::HostPacketHandle for PluginHostState {
    async fn id(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<i32> {
        Ok(packet_handle_from_resource(self, &packet)?.event.packet.id)
    }

    async fn set_id(&mut self, packet: Resource<PacketHandle>, id: i32) -> wasmtime::Result<()> {
        packet_handle_from_resource_mut(self, &packet)?
            .event
            .packet
            .id = id;
        Ok(())
    }

    async fn direction(
        &mut self,
        packet: Resource<PacketHandle>,
    ) -> wasmtime::Result<PacketDirection> {
        Ok(to_wasm_direction(
            packet_handle_from_resource(self, &packet)?.event.direction,
        ))
    }

    async fn state(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<ConnectionState> {
        Ok(to_wasm_state(
            packet_handle_from_resource(self, &packet)?.event.state,
        ))
    }

    async fn is_java(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<bool> {
        Ok(matches!(
            packet_handle_from_resource(self, &packet)?.event.state,
            crate::plugin::packet::PacketConnectionState::Java(_)
        ))
    }

    async fn is_bedrock(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<bool> {
        Ok(matches!(
            packet_handle_from_resource(self, &packet)?.event.state,
            crate::plugin::packet::PacketConnectionState::Bedrock(_)
        ))
    }

    async fn is_cancelled(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<bool> {
        Ok(packet_handle_from_resource(self, &packet)?.event.cancelled)
    }

    async fn cancel(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<()> {
        packet_handle_from_resource_mut(self, &packet)?
            .event
            .cancelled = true;
        Ok(())
    }

    async fn protocol_version(
        &mut self,
        packet: Resource<PacketHandle>,
    ) -> wasmtime::Result<Option<u32>> {
        Ok(packet_handle_from_resource(self, &packet)?
            .event
            .player
            .as_ref()
            .map(|player| match &player.client {
                crate::net::ClientPlatform::Java(java) => {
                    java.version.load().protocol_version() as u32
                }
                crate::net::ClientPlatform::Bedrock(_) => {
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL
                }
            }))
    }

    async fn minecraft_version(
        &mut self,
        packet: Resource<PacketHandle>,
    ) -> wasmtime::Result<Option<String>> {
        Ok(self
            .protocol_version(packet)
            .await?
            .map(protocol_to_mc_version)
            .map(|version| version.to_string()))
    }

    async fn java_packet_key(
        &mut self,
        packet: Resource<PacketHandle>,
    ) -> wasmtime::Result<Option<JavaPacketKey>> {
        let packet = packet_handle_from_resource(self, &packet)?;
        let Some(player) = packet.event.player.as_ref() else {
            return Ok(None);
        };
        let crate::plugin::packet::PacketConnectionState::Java(_) = packet.event.state else {
            return Ok(None);
        };
        let version = match &player.client {
            crate::net::ClientPlatform::Java(java) => {
                protocol_to_mc_version(java.version.load().protocol_version() as u32)
            }
            crate::net::ClientPlatform::Bedrock(_) => return Ok(None),
        };
        let name =
            java_packet_catalog().get_name(version, packet.event.direction, packet.event.packet.id);
        Ok(name.map(|name| to_wasm_key(packet.event.direction, name.phase, name.name)))
    }

    async fn java_packet_id(
        &mut self,
        packet: Resource<PacketHandle>,
        phase: String,
        name: String,
    ) -> wasmtime::Result<Option<i32>> {
        let packet = packet_handle_from_resource(self, &packet)?;
        let Some(player) = packet.event.player.as_ref() else {
            return Ok(None);
        };
        let version = match &player.client {
            crate::net::ClientPlatform::Java(java) => {
                protocol_to_mc_version(java.version.load().protocol_version() as u32)
            }
            crate::net::ClientPlatform::Bedrock(_) => return Ok(None),
        };
        Ok(java_packet_catalog().get_id(version, packet.event.direction, &phase, &name))
    }

    async fn java_packet_id_for_key(
        &mut self,
        packet: Resource<PacketHandle>,
        key: JavaPacketKey,
    ) -> wasmtime::Result<Option<i32>> {
        self.java_packet_id(packet, key.phase, key.name).await
    }

    async fn payload(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<Vec<u8>> {
        Ok(packet_handle_from_resource(self, &packet)?
            .event
            .packet
            .payload
            .to_vec())
    }

    async fn payload_len(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<u32> {
        Ok(packet_handle_from_resource(self, &packet)?
            .event
            .packet
            .payload
            .len() as u32)
    }

    async fn replace_payload(
        &mut self,
        packet: Resource<PacketHandle>,
        payload: Vec<u8>,
    ) -> wasmtime::Result<()> {
        packet_handle_from_resource_mut(self, &packet)?
            .event
            .packet
            .payload = Bytes::from(payload);
        Ok(())
    }

    async fn reader(
        &mut self,
        packet: Resource<PacketHandle>,
    ) -> wasmtime::Result<Resource<PacketReader>> {
        let payload = packet_handle_from_resource(self, &packet)?
            .event
            .packet
            .payload
            .to_vec();
        self.add_packet_reader(PluginPacketReader {
            buf: payload,
            pos: 0,
        })
    }

    async fn reply_system_message(
        &mut self,
        packet: Resource<PacketHandle>,
        text: Resource<TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<bool> {
        let packet = packet_handle_from_resource(self, &packet)?;
        let Some(player) = packet.event.player else {
            return Ok(false);
        };
        let text = super::player::text_component_from_resource(self, &text);
        if self.should_defer_effects() {
            self.defer_effect(
                crate::plugin::loader::wasm::wasm_host::state::PendingEffect::PlayerSystemMessage {
                    player,
                    text,
                    overlay,
                },
            );
            return Ok(true);
        }
        player.send_system_message_raw(&text, overlay).await;
        Ok(true)
    }

    async fn reply_raw_packet(
        &mut self,
        packet: Resource<PacketHandle>,
        raw_packet: RawPacket,
    ) -> wasmtime::Result<bool> {
        let packet = packet_handle_from_resource(self, &packet)?;
        let Some(player) = packet.event.player else {
            return Ok(false);
        };
        if self.should_defer_effects() {
            self.defer_effect(
                crate::plugin::loader::wasm::wasm_host::state::PendingEffect::PlayerRawPacket {
                    player,
                    id: raw_packet.id,
                    payload: raw_packet.payload,
                },
            );
            return Ok(true);
        }
        match &player.client {
            crate::net::ClientPlatform::Java(java) => {
                let mut buf = Vec::new();
                pumpkin_protocol::codec::var_int::VarInt(raw_packet.id)
                    .encode(&mut buf)
                    .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
                buf.extend_from_slice(&raw_packet.payload);
                java.enqueue_packet_data(buf.into()).await;
            }
            crate::net::ClientPlatform::Bedrock(bedrock) => {
                bedrock
                    .send_raw_game_packet(raw_packet.id, raw_packet.payload)
                    .await
                    .map_err(|err| wasmtime::Error::msg(err.to_string()))?;
            }
        }
        Ok(true)
    }

    async fn reply_packet(
        &mut self,
        packet: Resource<PacketHandle>,
        custom_packet: Packet,
    ) -> wasmtime::Result<bool> {
        let packet = packet_handle_from_resource(self, &packet)?;
        let Some(player) = packet.event.player else {
            return Ok(false);
        };
        if self.should_defer_effects() {
            self.defer_effect(
                crate::plugin::loader::wasm::wasm_host::state::PendingEffect::PlayerCustomPayload {
                    player,
                    channel: custom_packet.channel,
                    data: custom_packet.data,
                },
            );
            return Ok(true);
        }
        player
            .send_custom_payload(&custom_packet.channel, &custom_packet.data)
            .await;
        Ok(true)
    }

    async fn decode_java_registered_schema(
        &mut self,
        packet: Resource<PacketHandle>,
    ) -> wasmtime::Result<Result<Vec<NamedFieldValue>, PacketSchemaError>> {
        let packet = packet_handle_from_resource(self, &packet)?;
        let Some(player) = packet.event.player.as_ref() else {
            return Ok(Err(PacketSchemaError::MissingSchema));
        };
        let crate::plugin::packet::PacketConnectionState::Java(_) = packet.event.state else {
            return Ok(Err(PacketSchemaError::MissingSchema));
        };
        let version = match &player.client {
            crate::net::ClientPlatform::Java(java) => {
                protocol_to_mc_version(java.version.load().protocol_version() as u32)
            }
            crate::net::ClientPlatform::Bedrock(_) => {
                return Ok(Err(PacketSchemaError::MissingSchema));
            }
        };

        let Some(name) =
            java_packet_catalog().get_name(version, packet.event.direction, packet.event.packet.id)
        else {
            return Ok(Err(PacketSchemaError::MissingSchema));
        };

        let direction = to_wasm_direction(packet.event.direction);
        let Some(schema) =
            java_packet_schema_registry().get(&schema_key(direction, &name.phase, &name.name))
        else {
            return Ok(Err(PacketSchemaError::MissingSchema));
        };

        Ok(schema.decode(packet.event.packet.payload.as_ref()))
    }

    async fn drop(&mut self, packet: Resource<PacketHandle>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<PacketHandleResource>(Resource::new_own(packet.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
