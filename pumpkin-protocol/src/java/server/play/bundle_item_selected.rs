use pumpkin_data::packet::serverbound::PLAY_BUNDLE_ITEM_SELECTED;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[java_packet(PLAY_BUNDLE_ITEM_SELECTED)]
pub struct SBundleItemSelected {
    pub slot_id: VarInt,
    pub selected_item_index: VarInt,
}
