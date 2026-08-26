// Last verified for v2169

use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(75)]
pub struct CShowCredits {
    pub player_runtime_id: VarULong,
    pub credits_state: VarInt,
}
