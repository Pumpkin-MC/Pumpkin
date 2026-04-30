use pumpkin_plugin_api::packet_ids_full;
use pumpkin_util::version::MinecraftVersion;

#[test]
fn chat_ids_match_supported_versions() {
    let chat = packet_ids_full::serverbound::PLAY_CHAT;
    let cmd = packet_ids_full::serverbound::PLAY_CHAT_COMMAND;
    let cmd_signed = packet_ids_full::serverbound::PLAY_CHAT_COMMAND_SIGNED;

    assert_eq!(cmd.to_id(MinecraftVersion::V_1_21), 0x04);
    assert_eq!(cmd_signed.to_id(MinecraftVersion::V_1_21), 0x05);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21), 0x06);

    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_2), 0x07);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_4), 0x07);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_5), 0x07);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_6), 0x08);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_7), 0x08);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_9), 0x08);
    assert_eq!(chat.to_id(MinecraftVersion::V_1_21_11), 0x08);
    assert_eq!(chat.to_id(MinecraftVersion::V_26_1), 0x09);
}
