use pumpkin_plugin_api::packet_ids_full;
use pumpkin_util::version::MinecraftVersion;

#[test]
fn full_packet_table_matches_chat_ids() {
    let full = packet_ids_full::serverbound::PLAY_CHAT;
    assert_eq!(full.to_id(MinecraftVersion::V_1_21), 0x06);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_2), 0x07);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_4), 0x07);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_5), 0x07);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_6), 0x08);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_7), 0x08);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_9), 0x08);
    assert_eq!(full.to_id(MinecraftVersion::V_1_21_11), 0x08);
    assert_eq!(full.to_id(MinecraftVersion::V_26_1), 0x09);
}
