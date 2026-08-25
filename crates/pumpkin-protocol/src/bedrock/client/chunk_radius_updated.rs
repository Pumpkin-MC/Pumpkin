// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(70)]
pub struct CChunkRadiusUpdated {
    pub chunk_radius: VarInt,
}
