use crate::{
    codec::{var_int::VarInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(75)]
pub struct CShowCredits {
    pub player_runtime_id: VarULong,
    pub status: VarInt,
}

impl CShowCredits {
    #[must_use]
    pub const fn new(player_runtime_id: VarULong, status: VarInt) -> Self {
        Self {
            player_runtime_id,
            status,
        }
    }
}
