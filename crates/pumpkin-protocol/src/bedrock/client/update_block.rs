// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(21)]
pub struct CUpdateBlock {
    pub block_position: BlockPos,
    pub block_runtime_id: VarUInt,
    pub flags: VarUInt,
    pub layer: VarUInt,
}

impl CUpdateBlock {
    #[must_use]
    pub const fn new(block_position: BlockPos, block_runtime_id: u32) -> Self {
        Self::with_layer(block_position, block_runtime_id, 0)
    }

    #[must_use]
    pub const fn with_layer(block_position: BlockPos, block_runtime_id: u32, layer: u32) -> Self {
        Self {
            block_position,
            block_runtime_id: VarUInt(block_runtime_id),
            flags: VarUInt(0x3), // neighbors | network
            layer: VarUInt(layer),
        }
    }
}
