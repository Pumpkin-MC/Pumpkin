use pumpkin_data::packet::serverbound::PLAY_SET_BEACON;
use pumpkin_macros::java_packet;
use serde::Deserialize;

use crate::codec::var_int::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_SET_BEACON)]
pub struct SSetBeacon {
    pub primary_effect: Option<VarInt>,
    pub secondary_effect: Option<VarInt>,
}
