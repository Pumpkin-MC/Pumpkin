use pumpkin_data::packet::serverbound::PLAY_USE_ITEM;
use pumpkin_macros::server_packet;
use serde::Deserialize;

use crate::VarInt;

#[derive(Deserialize)]
#[server_packet(PLAY_USE_ITEM)]
pub struct SUseItem {
    // 0 for main hand, 1 for off hand
    pub hand: VarInt,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}
