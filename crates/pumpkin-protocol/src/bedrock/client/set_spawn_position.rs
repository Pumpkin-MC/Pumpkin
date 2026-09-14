// Last verified for v2169

use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(Clone, Copy, PacketWrite)]
#[packet(43)]
pub struct CSetSpawnPosition {
    pub spawn_position_type: SpawnPositionType,
    pub block_position: BlockPos,
    pub dimension_type: VarInt,
    pub spawn_block_pos: BlockPos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SpawnPositionType {
    PlayerRespawn,
    WorldRespawn,
}

impl PacketWrite for SpawnPositionType {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(*self as i32).write(writer)
    }
}
