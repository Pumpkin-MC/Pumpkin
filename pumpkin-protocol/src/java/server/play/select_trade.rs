use pumpkin_data::packet::serverbound::PLAY_SELECT_TRADE;
use pumpkin_macros::java_packet;

use crate::VarInt;

/// Sent when the player selects a trade in a villager's trading UI.
#[derive(serde::Deserialize, serde::Serialize)]
#[java_packet(PLAY_SELECT_TRADE)]
pub struct SSelectTrade {
    pub selected_slot: VarInt,
}
