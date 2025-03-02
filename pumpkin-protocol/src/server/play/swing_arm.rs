use pumpkin_data::packet::serverbound::PLAY_SWING;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::VarInt;

#[derive(serde::Deserialize, Serialize)]
#[packet(PLAY_SWING)]
pub struct SSwingArm {
    pub hand: VarInt,
}
