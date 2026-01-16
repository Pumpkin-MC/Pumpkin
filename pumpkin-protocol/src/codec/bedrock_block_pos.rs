use std::io::{Error, Write};

use pumpkin_util::math::position::BlockPos;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

/// Bedrocks Writes and Reads BlockPos types in Packets differently
pub struct NetworkPos(pub BlockPos);

impl NetworkPos {
    pub fn write_signed<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.0.0.x).write(writer)?;
        VarInt(self.0.0.y).write(writer)?;
        VarInt(self.0.0.z).write(writer)
    }
}

impl PacketWrite for NetworkPos {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.0.0.x).write(writer)?;
        VarUInt(self.0.0.y as u32).write(writer)?;
        VarInt(self.0.0.z).write(writer)
    }
}
