use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_SET_CHUNK_CACHE_CENTER;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Updates the center of the player's loaded chunk radius (the "view center").
///
/// This packet tells the client which chunk coordinate it should use as the
/// focal point for loading and unloading chunks. It is typically sent when
/// a player moves across a chunk boundary.
#[java_packet(PLAY_SET_CHUNK_CACHE_CENTER)]
pub struct CCenterChunk {
    /// The X coordinate of the center chunk.
    pub chunk_x: VarInt,
    /// The Z coordinate of the center chunk.
    pub chunk_z: VarInt,
}

impl ClientPacket for CCenterChunk {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.chunk_x)?;
        write.write_var_int(&self.chunk_z)?;
        Ok(())
    }
}
