use std::io::{Error, ErrorKind, Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::var_uint::VarUInt,
    serial::{PacketRead, PacketWrite},
};

/// Sent by the server to transfer a single chunk of compressed resource pack data to the client.
#[packet(83)]
pub struct CResourcePackChunkData {
    /// Identifier of the pack (e.g. `UUID` or `UUID_version`).
    pub pack_id: String,
    /// 0-indexed sequential chunk number.
    pub chunk_index: u32,
    /// Byte offset within the total resource pack archive from which this chunk begins.
    pub data_offset: u64,
    /// Raw byte payload of the chunk.
    pub data: Vec<u8>,
}

impl PacketWrite for CResourcePackChunkData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.pack_id.write(writer)?;
        self.chunk_index.write(writer)?;
        self.data_offset.write(writer)?;
        VarUInt(self.data.len() as u32).write(writer)?;
        writer.write_all(&self.data)
    }
}

impl PacketRead for CResourcePackChunkData {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let pack_id = String::read(reader)?;
        let chunk_index = u32::read(reader)?;
        let data_offset = u64::read(reader)?;
        let len = VarUInt::read(reader)?.0 as usize;
        if len > 4 * 1024 * 1024 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Resource pack chunk data size {len} exceeds limit"),
            ));
        }
        let mut data = vec![0u8; len];
        reader.read_exact(&mut data)?;
        Ok(Self {
            pack_id,
            chunk_index,
            data_offset,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let packet = CResourcePackChunkData {
            pack_id: "test-uuid-123_1.0.0".to_string(),
            chunk_index: 2,
            data_offset: 2097152,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8],
        };

        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();

        let decoded = CResourcePackChunkData::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded.pack_id, packet.pack_id);
        assert_eq!(decoded.chunk_index, packet.chunk_index);
        assert_eq!(decoded.data_offset, packet.data_offset);
        assert_eq!(decoded.data, packet.data);
    }
}
