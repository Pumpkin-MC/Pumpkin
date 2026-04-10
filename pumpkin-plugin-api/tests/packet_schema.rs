use std::collections::BTreeMap;

use pumpkin_plugin_api::packet::java::{
    schema::{FieldType, FieldValue, PacketSchema, java_packet_schema_registry},
    typed::JavaPacketKey,
};

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
