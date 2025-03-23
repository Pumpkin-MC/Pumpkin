use pumpkin_data::packet::serverbound::PLAY_USE_ITEM;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_USE_ITEM)]
pub struct SUseItem {
    // 0 for main hand, 1 for off hand
    pub hand: VarInt,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}
