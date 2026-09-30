use std::io::Write;

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
    _version: &JavaMinecraftVersion,
) -> Result<(), WritingError> {
    if nbt_data.is_empty() || nbt_data == [0] {
        write.write_u8(0)?;
    } else {
        // In 1.20.2+, the root compound tag is unnamed.
        write.write_all(nbt_data).map_err(WritingError::IoError)?;
    }
    Ok(())
}

pub fn read_nbt_payload(
    bytebuf: &mut &[u8],
    _version: &JavaMinecraftVersion,
) -> Result<Box<[u8]>, ReadingError> {
    if bytebuf.is_empty() || bytebuf[0] == 0 {
        if !bytebuf.is_empty() {
            let _ = bytebuf.get_u8()?;
        }
        Ok(Box::new([]))
    } else {
        let all = bytebuf.to_vec().into_boxed_slice();
        *bytebuf = &[];
        Ok(all)
    }
}

impl ClientPacket for CBlockEntityData {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_block_pos(&self.location, version)?;

        write.write_var_int(&self.r#type)?;

        write_nbt_payload(&mut write, &self.nbt_data, version)
    }
}

impl<'a> ServerPacket<'a> for CBlockEntityData {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let location = bytebuf.get_block_pos(version)?;
        let r#type = bytebuf.get_var_int()?;
        let nbt_data = read_nbt_payload(bytebuf, version)?;
        Ok(Self {
            location,
            r#type,
            nbt_data,
        })
    }
}
