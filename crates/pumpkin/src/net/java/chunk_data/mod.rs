pub mod light;
pub mod util;
pub mod v1_18;

pub use light::ChunkLightExt;

use pumpkin_data::packet::clientbound::play::{CHUNKS_BIOMES, LEVEL_CHUNK_WITH_LIGHT};
use pumpkin_protocol::ClientPacket;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::packet::MultiVersionJavaPacket;
use pumpkin_protocol::ser::{NetworkWriteExt, WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use std::io::Write;

/// Sent by the server to provide the client with the full data for a chunk.
///
/// This includes heightmaps, the actual block and biome data (organized into sections),
/// block entities (like signs or chests), and the light level information for both
/// sky and block light.
pub struct CChunkData<'a>(pub &'a ChunkData);

impl MultiVersionJavaPacket for CChunkData<'_> {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        LEVEL_CHUNK_WITH_LIGHT.to_id(version)
    }
}

impl<'a> CChunkData<'a> {
    #[must_use]
    pub const fn new(chunk: &'a ChunkData) -> Self {
        Self(chunk)
    }
}

impl ClientPacket for CChunkData<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        v1_18::write_chunk_data(self.0, write, version)
    }
}

/// Replaces the biomes of chunks the client already has, without resending
/// their blocks (`CHUNKS_BIOMES`).
pub struct CChunksBiomes<'a>(pub &'a [&'a ChunkData]);

impl MultiVersionJavaPacket for CChunksBiomes<'_> {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        CHUNKS_BIOMES.to_id(version)
    }
}

impl ClientPacket for CChunksBiomes<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.0.len() as i32))?;
        for chunk in self.0 {
            let biome_sections =
                chunk.section.biome_sections.read().map_err(|_| {
                    WritingError::Message("biome_sections read lock poisoned".into())
                })?;
            let mut data = Vec::new();
            for biomes in biome_sections.iter() {
                v1_18::write_biomes(&mut data, biomes, version)?;
            }
            // A chunk position is one long with z in the high half.
            write.write_i32_be(chunk.z)?;
            write.write_i32_be(chunk.x)?;
            write.write_var_int(&VarInt(data.len() as i32))?;
            write.write_slice(&data)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_world::chunk::ChunkData;

    /// The client reads each entry as a packed chunk position (z in the high
    /// half) and a length-prefixed byte array; get either wrong and every
    /// following entry is garbage.
    #[test]
    fn chunks_biomes_entry_layout() {
        let chunk = ChunkData::empty(3, -2);
        let mut buf = Vec::new();
        CChunksBiomes(&[&chunk])
            .write_packet_data(&mut buf, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        assert_eq!(buf[0], 1, "one entry");
        assert_eq!(&buf[1..5], &(-2i32).to_be_bytes(), "z comes first");
        assert_eq!(&buf[5..9], &3i32.to_be_bytes());
        let mut rest = &buf[9..];
        let len = pumpkin_protocol::ser::NetworkReadExt::get_var_int(&mut rest).unwrap();
        assert_eq!(
            len.0 as usize,
            rest.len(),
            "the length covers exactly the biome data"
        );
        assert!(!rest.is_empty());
    }

    #[test]
    fn chunk_data_all_versions() {
        let chunk = ChunkData::empty(0, 0);
        let packet = CChunkData(&chunk);

        let versions = [JavaMinecraftVersion::V_26_3];

        for version in versions {
            let mut buf = Vec::new();
            let id = CChunkData::to_id(version);
            assert_ne!(id, -1, "Packet ID for version {version:?} must be valid");
            assert!(
                packet.write_packet_data(&mut buf, &version).is_ok(),
                "Failed to serialize chunk data for version {version:?}"
            );
            assert!(
                !buf.is_empty(),
                "Serialized buffer must not be empty for version {version:?}"
            );
        }
    }

    #[test]
    fn populated_chunk_data_all_versions() {
        let chunk = ChunkData::empty(0, 0);
        chunk
            .section
            .set_block_absolute_y(0, 64, 0, pumpkin_data::Block::STONE.default_state.id);
        chunk
            .section
            .set_block_absolute_y(1, 64, 1, pumpkin_data::Block::DIRT.default_state.id);

        let mut nbt = pumpkin_nbt::compound::NbtCompound::new();
        nbt.put_string("id", "minecraft:chest".to_string());
        chunk.pending_block_entities.lock().unwrap().insert(
            pumpkin_util::math::position::BlockPos(pumpkin_util::math::vector3::Vector3::new(
                0, 64, 0,
            )),
            nbt,
        );

        let packet = CChunkData(&chunk);

        let versions = [JavaMinecraftVersion::V_26_3];

        for version in versions {
            let mut buf = Vec::new();
            let id = CChunkData::to_id(version);
            assert_ne!(id, -1, "Packet ID for version {version:?} must be valid");
            assert!(
                packet.write_packet_data(&mut buf, &version).is_ok(),
                "Failed to serialize populated chunk data for version {version:?}"
            );
            assert!(
                !buf.is_empty(),
                "Serialized buffer must not be empty for version {version:?}"
            );
        }
    }
}
