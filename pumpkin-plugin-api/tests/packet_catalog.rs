use pumpkin_plugin_api::packet::{PacketDirection, java::catalog::java_packet_catalog};
use pumpkin_util::version::MinecraftVersion;

#[test]
fn catalog_resolves_ids_for_supported_versions() {
    let catalog = java_packet_catalog();

    assert_eq!(
        catalog.get_id(MinecraftVersion::V_1_21, PacketDirection::Serverbound, "play", "chat"),
        Some(0x06)
    );
    assert_eq!(
        catalog.get_id(
            MinecraftVersion::V_1_21_2,
            PacketDirection::Serverbound,
            "play",
            "chat"
        ),
        Some(0x07)
    );
    assert_eq!(
        catalog.get_id(
            MinecraftVersion::V_1_21_11,
            PacketDirection::Serverbound,
            "play",
            "chat"
        ),
        Some(0x08)
    );
    assert_eq!(
        catalog.get_id(MinecraftVersion::V_26_1, PacketDirection::Serverbound, "play", "chat"),
        Some(0x09)
    );
}

#[test]
fn catalog_reverse_lookup_works() {
    let catalog = java_packet_catalog();

    let packet = catalog
        .get_name(MinecraftVersion::V_26_1, PacketDirection::Serverbound, 0x09)
        .expect("packet name should exist");

    assert_eq!(packet.phase, "play");
    assert_eq!(packet.name, "chat");
}

#[test]
fn catalog_lists_all_packets_for_direction() {
    let catalog = java_packet_catalog();
    let packets = catalog.list(MinecraftVersion::V_26_1, PacketDirection::Serverbound);

    assert!(packets.iter().any(|packet| {
        packet.phase == "play" && packet.name == "chat" && packet.id == 0x09
    }));
    assert!(packets.len() > 20);
}
