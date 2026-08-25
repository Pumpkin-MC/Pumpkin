use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[packet(82)]
pub struct CResourcePackDataInfo<'a> {
    pub resource_name: &'a str,
    pub chunk_size: u32,
    pub number_of_chunks: u32,
    pub file_size: u64,
    pub file_hash: &'a [u8],
    pub is_premium_pack: bool,
    pub pack_type: u8,
}

impl PacketWrite for CResourcePackDataInfo<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.resource_name.write(writer)?;
        self.chunk_size.write(writer)?;
        self.number_of_chunks.write(writer)?;
        self.file_size.write(writer)?;
        VarUInt(self.file_hash.len() as u32).write(writer)?;
        writer.write_all(self.file_hash)?;
        self.is_premium_pack.write(writer)?;
        self.pack_type.write(writer)
    }
}
