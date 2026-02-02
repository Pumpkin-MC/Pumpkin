use pumpkin_data::packet::serverbound::PLAY_SWING;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[java_packet(PLAY_SWING)]
pub struct SSwingArm {
    pub hand: VarInt,
}
