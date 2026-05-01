use pumpkin_data::packet::serverbound::PLAY_RENAME_ITEM;
use pumpkin_macros::java_packet;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[java_packet(PLAY_RENAME_ITEM)]
pub struct SRenameItem {
    pub item_name: String,
}
