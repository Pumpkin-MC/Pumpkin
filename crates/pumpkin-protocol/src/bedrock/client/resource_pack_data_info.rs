use std::io::{Error, ErrorKind, Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::var_uint::VarUInt,
    serial::{PacketRead, PacketWrite},
};

/// Information describing a resource pack to be streamed in chunks to the client.
#[packet(82)]
pub struct CResourcePackDataInfo {
    /// Identifier of the pack (e.g. `UUID` or `UUID_version`).
    pub pack_id: String,
    /// Maximum size of chunks in bytes (typically 1 MB = 1,048,576).
    pub max_chunk_size: u32,
    /// Total number of chunks the resource pack is divided into.
    pub chunk_count: u32,
    /// Total size of the compressed pack archive in bytes.
    pub compressed_pack_size: u64,
    /// SHA-256 hash of the pack archive (32 bytes).
    pub file_hash: Vec<u8>,
    /// Whether the pack was bought from the Minecraft Marketplace.
    pub is_premium: bool,
    /// Type of the resource pack (e.g. 1 for Addon, 6 for Resources).
    pub pack_type: u8,
}

impl CResourcePackDataInfo {
    pub const TYPE_INVALID: u8 = 0;
    pub const TYPE_ADDON: u8 = 1;
    pub const TYPE_CACHED: u8 = 2;
    pub const TYPE_COPY_PROTECTED: u8 = 3;
    pub const TYPE_BEHAVIORS: u8 = 4;
    pub const TYPE_PERSONA_PIECE: u8 = 5;
    pub const TYPE_RESOURCES: u8 = 6;
    pub const TYPE_SKINS: u8 = 7;
    pub const TYPE_WORLD_TEMPLATE: u8 = 8;
}

impl PacketWrite for CResourcePackDataInfo {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.pack_id.write(writer)?;
        self.max_chunk_size.write(writer)?;
        self.chunk_count.write(writer)?;
        self.compressed_pack_size.write(writer)?;
        VarUInt(self.file_hash.len() as u32).write(writer)?;
        writer.write_all(&self.file_hash)?;
        self.is_premium.write(writer)?;
        self.pack_type.write(writer)
    }
}

impl PacketRead for CResourcePackDataInfo {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let pack_id = String::read(reader)?;
        let max_chunk_size = u32::read(reader)?;
        let chunk_count = u32::read(reader)?;
        let compressed_pack_size = u64::read(reader)?;
        let hash_len = VarUInt::read(reader)?.0 as usize;
        if hash_len > 64 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Resource pack hash too long",
            ));
        }
        let mut file_hash = vec![0u8; hash_len];
        reader.read_exact(&mut file_hash)?;
        let is_premium = bool::read(reader)?;
        let pack_type = u8::read(reader)?;
        Ok(Self {
            pack_id,
            max_chunk_size,
            chunk_count,
            compressed_pack_size,
            file_hash,
            is_premium,
            pack_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let packet = CResourcePackDataInfo {
            pack_id: "test-uuid-123_1.0.0".to_string(),
            max_chunk_size: 1048576,
            chunk_count: 5,
            compressed_pack_size: 5000000,
            file_hash: vec![0xab; 32],
            is_premium: false,
            pack_type: CResourcePackDataInfo::TYPE_RESOURCES,
        };

        let mut bytes = Vec::new();
        packet.write(&mut bytes).unwrap();

        let decoded = CResourcePackDataInfo::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(decoded.pack_id, packet.pack_id);
        assert_eq!(decoded.max_chunk_size, packet.max_chunk_size);
        assert_eq!(decoded.chunk_count, packet.chunk_count);
        assert_eq!(decoded.compressed_pack_size, packet.compressed_pack_size);
        assert_eq!(decoded.file_hash, packet.file_hash);
        assert_eq!(decoded.is_premium, packet.is_premium);
        assert_eq!(decoded.pack_type, packet.pack_type);
    }
}
