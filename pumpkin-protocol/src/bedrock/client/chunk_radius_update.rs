use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(70)]
pub struct CChunkRadiusUpdate {
    pub chunk_radius: VarInt,
}
