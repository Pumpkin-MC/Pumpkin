use pumpkin_data::packet::serverbound::PLAY_ATTACK;
use pumpkin_macros::java_packet;
use serde::Deserialize;

use crate::codec::var_int::VarInt;

#[derive(Deserialize)]
#[java_packet(PLAY_ATTACK)]
pub struct SAttack {
    pub entity_id: VarInt,
}
