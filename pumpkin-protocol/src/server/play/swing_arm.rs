use pumpkin_data::packet::serverbound::PLAY_SWING;
use pumpkin_macros::server_packet;

use crate::VarInt;

#[derive(serde::Deserialize)]
#[server_packet(PLAY_SWING)]
pub struct SSwingArm {
    pub hand: VarInt,
}
