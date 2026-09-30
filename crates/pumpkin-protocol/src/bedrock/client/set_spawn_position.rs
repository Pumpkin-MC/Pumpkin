// Last verified for v2169

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(i32)]
#[serial(varint)]
pub enum SpawnPositionType {
    PlayerRespawn,
    WorldRespawn,
}
