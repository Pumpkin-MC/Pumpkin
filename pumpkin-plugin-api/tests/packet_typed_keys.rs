use pumpkin_plugin_api::packet::{PacketDirection, java::typed::JavaPacketKey};
use pumpkin_util::version::MinecraftVersion;

#[test]
fn generated_key_can_be_resolved_from_parts() {
    let key = JavaPacketKey::from_parts(PacketDirection::Serverbound, "play", "chat")
        .expect("key should exist");

    assert_eq!(key.direction(), PacketDirection::Serverbound);
    assert_eq!(key.phase(), "play");
    assert_eq!(key.name(), "chat");
    assert_eq!(key.id_for_version(MinecraftVersion::V_26_1), 0x09);
}

#[test]
fn generated_key_list_contains_many_packets() {
    let all = JavaPacketKey::all();
    assert!(all.len() > 100);
}
