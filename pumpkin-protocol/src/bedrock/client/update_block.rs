use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[packet(21)]
pub struct CUpdateBlock {
    pub position: BlockPos,
    pub block_runtime_id: VarUInt,
    pub flags: VarUInt,
    pub layer: VarUInt,
}

impl CUpdateBlock {
    #[must_use]
    pub const fn new(position: BlockPos, block_runtime_id: u32) -> Self {
        Self {
            position,
            block_runtime_id: VarUInt(block_runtime_id),
            flags: VarUInt(0x3), // neighbors | network
            layer: VarUInt(0),
        }
    }
}

impl PacketWrite for CUpdateBlock {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.position.write(writer)?;
        self.block_runtime_id.write(writer)?;
        self.flags.write(writer)?;
        self.layer.write(writer)?;
        Ok(())
    }
}
