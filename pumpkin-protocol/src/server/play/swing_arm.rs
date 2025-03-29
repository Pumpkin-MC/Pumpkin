use pumpkin_data::packet::serverbound::PLAY_SWING;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Deserialize, Serialize)]
#[packet(PLAY_SWING)]
pub struct SSwingArm {
    pub hand: VarInt,
}
