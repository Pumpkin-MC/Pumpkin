use pumpkin_data::packet::serverbound::PLAY_RENAME_ITEM;
use pumpkin_macros::java_packet;

/// Sent when the player types in the anvil rename text field.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_RENAME_ITEM)]
pub struct SRenameItem {
    pub item_name: String,
}
