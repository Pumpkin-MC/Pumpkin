use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[packet(83)]
pub struct CResourcePackChunkData<'a> {
    pub resource_name: &'a str,
    pub chunk_id: u32,
    pub byte_offset: u64,
    pub chunk_data: &'a [u8],
}

impl PacketWrite for CResourcePackChunkData<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.resource_name.write(writer)?;
        self.chunk_id.write(writer)?;
        self.byte_offset.write(writer)?;
        VarUInt(self.chunk_data.len() as u32).write(writer)?;
        writer.write_all(self.chunk_data)
    }
}
