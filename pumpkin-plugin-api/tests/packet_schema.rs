use std::collections::BTreeMap;

use pumpkin_plugin_api::packet::java::{
    schema::{FieldType, FieldValue, PacketSchema, java_packet_schema_registry},
    typed::JavaPacketKey,
};
use pumpkin_plugin_api::packet::{BlockPos, Vec3I16};

#[test]
fn schema_roundtrip_works() {
    let schema = PacketSchema::new()
        .field("message", FieldType::String { max_len: 256 })
        .field("count", FieldType::VarInt)
        .field("flag", FieldType::Bool);

    let mut values = BTreeMap::new();
    values.insert(
        "message".to_string(),
        FieldValue::String("hello".to_string()),
    );
    values.insert("count".to_string(), FieldValue::VarInt(7));
    values.insert("flag".to_string(), FieldValue::Bool(true));

    let payload = schema.encode(&values).expect("encode");
    let decoded = schema.decode(&payload).expect("decode");

    assert_eq!(decoded, values);
}

#[test]
fn schema_roundtrip_supports_positions_vectors_and_optional_tail() {
    let schema = PacketSchema::new()
        .field("position", FieldType::BlockPos)
        .field("delta", FieldType::Vec3I16)
        .field_optional("tag", FieldType::String { max_len: 32 })
        .field("tail", FieldType::RemainingBytes);

    let mut values = BTreeMap::new();
    values.insert(
        "position".to_string(),
        FieldValue::BlockPos(BlockPos {
            x: 12,
            y: 64,
            z: -4,
        }),
    );
    values.insert(
        "delta".to_string(),
        FieldValue::Vec3I16(Vec3I16 { x: 1, y: -2, z: 3 }),
    );
    values.insert("tail".to_string(), FieldValue::Bytes(vec![1, 2, 3, 4]));

    let payload = schema.encode(&values).expect("encode");
    let decoded = schema.decode(&payload).expect("decode");

    assert_eq!(decoded, values);
}

#[test]
fn generated_schema_is_available_for_simple_packets() {
    let key = JavaPacketKey::from_parts(
        pumpkin_plugin_api::packet::PacketDirection::Serverbound,
        "play",
        "keep_alive",
    )
    .expect("keep_alive key exists");

    let guard = java_packet_schema_registry().read().expect("lock");
    let schema = guard.get(key).expect("generated schema exists");

    assert_eq!(
        schema,
        &PacketSchema::new().field("keep_alive_id", FieldType::I64)
    );
}

#[test]
fn generated_schema_is_available_for_vector_packet() {
    let key = JavaPacketKey::from_parts(
        pumpkin_plugin_api::packet::PacketDirection::Serverbound,
        "play",
        "move_player_pos",
    )
    .expect("move_player_pos key exists");

    let guard = java_packet_schema_registry().read().expect("lock");
    let schema = guard.get(key).expect("generated schema exists");

    assert_eq!(
        schema,
        &PacketSchema::new()
            .field("position", FieldType::Vec3F64)
            .field("collision", FieldType::U8)
    );
}

#[test]
fn schema_registry_stores_by_generated_key() {
    let key = JavaPacketKey::from_parts(
        pumpkin_plugin_api::packet::PacketDirection::Serverbound,
        "play",
        "chat",
    )
    .expect("chat key exists");

    let schema = PacketSchema::new().field("message", FieldType::String { max_len: 256 });

    {
        let mut guard = java_packet_schema_registry().write().expect("lock");
        guard.register(key, schema.clone());
    }

    let guard = java_packet_schema_registry().read().expect("lock");
    let loaded = guard.get(key).expect("schema registered");
    assert_eq!(loaded, &schema);
}
