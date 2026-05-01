use pumpkin_data::packet::serverbound::PLAY_SELECT_TRADE;
use pumpkin_macros::java_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_SELECT_TRADE)]
pub struct SSelectTrade {
    pub selected_slot: VarInt,
}
