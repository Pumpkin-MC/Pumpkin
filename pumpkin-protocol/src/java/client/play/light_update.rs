use crate::WritingError;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_LIGHT_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use std::io::Write;

use super::chunk_data::write_chunk_light_data;

/// Sent by the server to update light levels (block light and sky light) for a chunk.
///
/// This packet updates lighting data for a specific chunk without sending the full chunk data.
/// It's used when block placement or removal changes the lighting in a chunk.
#[java_packet(PLAY_LIGHT_UPDATE)]
pub struct CLightUpdate<'a>(pub &'a ChunkData);

impl ClientPacket for CLightUpdate<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_var_int(&VarInt(self.0.x))?;
        // Chunk Z
        write.write_var_int(&VarInt(self.0.z))?;

        write_chunk_light_data(self.0, write)
    }
}
