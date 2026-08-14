use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};

#[derive(Clone, Debug)]
pub struct CacheBlob {
    pub hash: u64,
    pub payload: Vec<u8>,
}

#[packet(136)]
pub struct CClientCacheMissResponse<'a> {
    pub blobs: &'a [CacheBlob],
}

impl PacketWrite for CClientCacheMissResponse<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.blobs.len() as u32).write(writer)?;
        for blob in self.blobs {
            writer.write_all(&blob.hash.to_le_bytes())?;
            VarUInt(blob.payload.len() as u32).write(writer)?;
            writer.write_all(&blob.payload)?;
        }
        Ok(())
    }
}
