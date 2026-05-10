use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_BLOCK_ENTITY_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::MinecraftVersion};

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

/// Updates the NBT data of a block entity (e.g., signs, chests, or banners).
///
/// This packet is sent by the server when a block entity's state changes
/// (like text on a sign) or when the block entity is loaded into the client's view.
#[java_packet(PLAY_BLOCK_ENTITY_DATA)]
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

impl ClientPacket for CBlockEntityData {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_block_pos(&self.location)?;

        write.write_var_int(&self.r#type)?;

        write
            .write_all(&self.nbt_data)
            .map_err(WritingError::IoError)
    }
}
