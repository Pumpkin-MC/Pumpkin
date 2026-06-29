use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::serial::PacketRead;

#[derive(Debug, PacketRead)]
#[packet(34)]
pub struct SBlockPickRequest {
    pub block_pos: BlockPos,
    pub add_block_nbt: bool,
    pub hotbar_slot: u8,
}
