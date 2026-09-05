use std::io::{Read, Write};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use pumpkin_data::block_entity_type_id_remap::remap_block_entity_type_id_for_version;
use pumpkin_data::packet::clientbound::play::BLOCK_ENTITY_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};

/// Updates the NBT data of a block entity (e.g., signs, chests, or banners).
///
/// This packet is sent by the server when a block entity's state changes
/// (like text on a sign) or when the block entity is loaded into the client's view.
#[java_packet(BLOCK_ENTITY_DATA)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CBlockEntityData {
    /// The world coordinates of the block entity.
    pub location: BlockPos,
    /// The type of block entity being updated (e.g., Mob Spawner, Command Block).
    pub r#type: VarInt,
    /// The raw NBT payload containing the block's specific data.
    pub nbt_data: Box<[u8]>,
}

impl CBlockEntityData {
    #[must_use]
    pub const fn new(location: BlockPos, r#type: VarInt, nbt_data: Box<[u8]>) -> Self {
        Self {
            location,
            r#type,
            nbt_data,
        }
    }
}

pub fn write_nbt_payload(
    mut write: impl Write,
    nbt_data: &[u8],
    version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if *version >= JavaMinecraftVersion::V_1_8 {
        if nbt_data.is_empty() || nbt_data == [0] {
            write.write_u8(0)?;
        } else if *version < JavaMinecraftVersion::V_1_20_2 {
            // In 1.8..1.20.1, the root compound tag is named.
            // If nbt_data is an unnamed compound (0x0A followed by content), insert empty name (0x00, 0x00).
            if nbt_data.len() >= 3 && nbt_data[0] == 0x0A && nbt_data[1] == 0 && nbt_data[2] == 0 {
                write.write_all(nbt_data).map_err(WritingError::IoError)?;
            } else if nbt_data[0] == 0x0A {
                write.write_u8(0x0A)?;
                write.write_u16_be(0)?;
                write
                    .write_all(&nbt_data[1..])
                    .map_err(WritingError::IoError)?;
            } else {
                write.write_all(nbt_data).map_err(WritingError::IoError)?;
            }
        } else {
            // In 1.20.2+, the root compound tag is unnamed.
            write.write_all(nbt_data).map_err(WritingError::IoError)?;
        }
    } else {
        // <= 1.7.6
        if nbt_data.is_empty() || nbt_data == [0] {
            write.write_i16_be(-1)?;
        } else {
            let mut named_bytes = Vec::with_capacity(nbt_data.len() + 2);
            if nbt_data.len() >= 3 && nbt_data[0] == 0x0A && nbt_data[1] == 0 && nbt_data[2] == 0 {
                named_bytes.extend_from_slice(nbt_data);
            } else if nbt_data[0] == 0x0A {
                named_bytes.push(0x0A);
                named_bytes.extend_from_slice(&[0x00, 0x00]);
                named_bytes.extend_from_slice(&nbt_data[1..]);
            } else {
                named_bytes.extend_from_slice(nbt_data);
            }

            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder
                .write_all(&named_bytes)
                .map_err(WritingError::IoError)?;
            let compressed = encoder.finish().map_err(WritingError::IoError)?;

            write.write_i16_be(compressed.len() as i16)?;
            write
                .write_all(&compressed)
                .map_err(WritingError::IoError)?;
        }
    }
    Ok(())
}

pub fn read_nbt_payload(
    bytebuf: &mut &[u8],
    version: &JavaMinecraftVersion,
) -> Result<Box<[u8]>, ReadingError> {
    if *version >= JavaMinecraftVersion::V_1_8 {
        if bytebuf.is_empty() || bytebuf[0] == 0 {
            if !bytebuf.is_empty() {
                let _ = bytebuf.get_u8()?;
            }
            Ok(Box::new([]))
        } else if *version < JavaMinecraftVersion::V_1_20_2 {
            let all = bytebuf.to_vec();
            *bytebuf = &[];
            if all.len() >= 3 && all[0] == 0x0A && all[1] == 0 && all[2] == 0 {
                let mut unnamed = Vec::with_capacity(all.len() - 2);
                unnamed.push(0x0A);
                unnamed.extend_from_slice(&all[3..]);
                Ok(unnamed.into_boxed_slice())
            } else {
                Ok(all.into_boxed_slice())
            }
        } else {
            let all = bytebuf.to_vec().into_boxed_slice();
            *bytebuf = &[];
            Ok(all)
        }
    } else {
        // <= 1.7.6
        let length = bytebuf.get_i16_be()?;
        if length <= 0 {
            Ok(Box::new([]))
        } else {
            if bytebuf.len() < length as usize {
                return Err(ReadingError::Incomplete(
                    "Not enough bytes for compressed NBT".into(),
                ));
            }
            let compressed = &bytebuf[..length as usize];
            *bytebuf = &bytebuf[length as usize..];
            let mut decoder = GzDecoder::new(compressed);
            let mut decompressed = Vec::new();
            decoder
                .read_to_end(&mut decompressed)
                .map_err(|e| ReadingError::Message(e.to_string()))?;
            if decompressed.len() >= 3
                && decompressed[0] == 0x0A
                && decompressed[1] == 0
                && decompressed[2] == 0
            {
                let mut unnamed = Vec::with_capacity(decompressed.len() - 2);
                unnamed.push(0x0A);
                unnamed.extend_from_slice(&decompressed[3..]);
                Ok(unnamed.into_boxed_slice())
            } else {
                Ok(decompressed.into_boxed_slice())
            }
        }
    }
}

impl ClientPacket for CBlockEntityData {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_block_pos(&self.location, version)?;

        let remapped_type = remap_block_entity_type_id_for_version(self.r#type.0 as u32, *version);
        if *version >= JavaMinecraftVersion::V_1_18 {
            write.write_var_int(&VarInt(remapped_type as i32))?;
        } else {
            write.write_u8(remapped_type as u8)?;
        }

        write_nbt_payload(&mut write, &self.nbt_data, version)
    }
}

impl<'a> ServerPacket<'a> for CBlockEntityData {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let location = bytebuf.get_block_pos(version)?;
        let r#type = if *version >= JavaMinecraftVersion::V_1_18 {
            bytebuf.get_var_int()?
        } else {
            VarInt(i32::from(bytebuf.get_u8()?))
        };
        let nbt_data = read_nbt_payload(bytebuf, version)?;
        Ok(Self {
            location,
            r#type,
            nbt_data,
        })
    }
}
