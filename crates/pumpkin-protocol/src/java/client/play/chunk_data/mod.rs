pub mod util;
pub mod v1_18;
pub mod v1_7;
pub mod v1_8;
pub mod v1_9;

use crate::packet::MultiVersionJavaPacket;
use crate::{ClientPacket, WritingError};
use pumpkin_data::packet::clientbound::play::LEVEL_CHUNK_WITH_LIGHT;
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
        if version >= &JavaMinecraftVersion::V_1_18 {
            v1_18::write_chunk_data(self.0, write, version)
        } else if version >= &JavaMinecraftVersion::V_1_9 {
            v1_9::write_chunk_data(self.0, write, version)
        } else if version == &JavaMinecraftVersion::V_1_8 {
            v1_8::write_chunk_data(self.0, write, version)
        } else if version == &JavaMinecraftVersion::V_1_7_2
            || version == &JavaMinecraftVersion::V_1_7_6
        {
            v1_7::write_chunk_data(self.0, write, version)
        } else {
            v1_18::write_chunk_data(self.0, write, version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_world::chunk::ChunkData;

    #[test]
    fn chunk_data_all_versions() {
        let chunk = ChunkData::empty(0, 0);
        let packet = CChunkData(&chunk);

        let versions = [
            JavaMinecraftVersion::V_1_7_2,
            JavaMinecraftVersion::V_1_7_6,
            JavaMinecraftVersion::V_1_8,
            JavaMinecraftVersion::V_1_9,
            JavaMinecraftVersion::V_1_12_2,
            JavaMinecraftVersion::V_1_13_2,
            JavaMinecraftVersion::V_1_14_4,
            JavaMinecraftVersion::V_1_15_2,
            JavaMinecraftVersion::V_1_16_1,
            JavaMinecraftVersion::V_1_16_4,
            JavaMinecraftVersion::V_1_17_1,
            JavaMinecraftVersion::V_1_18_2,
            JavaMinecraftVersion::V_1_19_4,
            JavaMinecraftVersion::V_1_20_2,
            JavaMinecraftVersion::V_1_21_4,
            JavaMinecraftVersion::V_1_21_5,
            JavaMinecraftVersion::V_26_1,
            JavaMinecraftVersion::V_26_2,
        ];

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

        let versions = [
            JavaMinecraftVersion::V_1_7_2,
            JavaMinecraftVersion::V_1_7_6,
            JavaMinecraftVersion::V_1_8,
            JavaMinecraftVersion::V_1_9,
            JavaMinecraftVersion::V_1_12_2,
            JavaMinecraftVersion::V_1_13_2,
            JavaMinecraftVersion::V_1_14_4,
            JavaMinecraftVersion::V_1_15_2,
            JavaMinecraftVersion::V_1_16_1,
            JavaMinecraftVersion::V_1_16_4,
            JavaMinecraftVersion::V_1_17_1,
            JavaMinecraftVersion::V_1_18_2,
            JavaMinecraftVersion::V_1_19_4,
            JavaMinecraftVersion::V_1_20_2,
            JavaMinecraftVersion::V_1_21_4,
            JavaMinecraftVersion::V_1_21_5,
            JavaMinecraftVersion::V_26_1,
            JavaMinecraftVersion::V_26_2,
        ];

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
