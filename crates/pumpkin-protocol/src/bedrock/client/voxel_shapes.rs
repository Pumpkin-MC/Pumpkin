use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[packet(337)]
pub struct CVoxelShapes;

impl PacketWrite for CVoxelShapes {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(0).write(writer)?;
        VarUInt(0).write(writer)?;
        0u16.write(writer)
    }
}
