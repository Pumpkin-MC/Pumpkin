use std::io::Write;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::packet::MultiVersionJavaPacket;
use crate::ser::{NetworkWriteExt, WritingError};
use pumpkin_data::packet::clientbound::play::LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::version::JavaMinecraftVersion;

use super::light_update::LightData;

/// Heightmap data accompanying a chunk packet.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ChunkHeightmaps {
    pub world_surface: Option<Vec<i64>>,
    pub motion_blocking: Option<Vec<i64>>,
    pub motion_blocking_no_leaves: Option<Vec<i64>>,
}

impl ChunkHeightmaps {
    pub fn write_to(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_21_5 {
            write.write_var_int(&VarInt(3))?; // Map size

            let mut write_heightmap = |index: i32, data: &[i64]| -> Result<(), WritingError> {
                write.write_var_int(&VarInt(index))?;
                write.write_var_int(&VarInt(data.len() as i32))?;
                for val in data {
                    write.write_i64_be(*val)?;
                }
                Ok(())
            };

            write_heightmap(1, self.world_surface.as_deref().unwrap_or(&[0; 37]))?;
            write_heightmap(4, self.motion_blocking.as_deref().unwrap_or(&[0; 37]))?;
            write_heightmap(
                5,
                self.motion_blocking_no_leaves
                    .as_deref()
                    .unwrap_or(&[0; 37]),
            )?;
        } else {
            let mut comp = NbtCompound::new();
            if let Some(ref ws) = self.world_surface {
                comp.put("WORLD_SURFACE", NbtTag::LongArray(ws.clone()));
            }
            if let Some(ref mb) = self.motion_blocking {
                comp.put("MOTION_BLOCKING", NbtTag::LongArray(mb.clone()));
            }
            if let Some(ref mbnl) = self.motion_blocking_no_leaves {
                comp.put("MOTION_BLOCKING_NO_LEAVES", NbtTag::LongArray(mbnl.clone()));
            }
            write.write_compound_nbt_with_version(Some(&comp), version)?;
        }
        Ok(())
    }
}

/// Block entity metadata serialized in chunk packets.
#[derive(Clone, Debug, PartialEq)]
pub struct ChunkBlockEntity {
    pub packed_xz: u8,
    pub y: i16,
    pub type_id: VarInt,
    pub data: NbtCompound,
}

impl ChunkBlockEntity {
    #[must_use]
    pub const fn new(packed_xz: u8, y: i16, type_id: VarInt, data: NbtCompound) -> Self {
        Self {
            packed_xz,
            y,
            type_id,
            data,
        }
    }
}

/// The chunk data and light packet sent to clients (`LEVEL_CHUNK_WITH_LIGHT`).
///
/// Contains chunk position, heightmaps, pre-encoded section and biome payload,
/// block entities, and lighting data.
#[derive(Clone, Debug, PartialEq)]
pub struct CChunkData<'a> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub heightmaps: ChunkHeightmaps,
    pub data: &'a [u8],
    pub block_entities: Vec<ChunkBlockEntity>,
    pub light_data: LightData,
}

impl MultiVersionJavaPacket for CChunkData<'_> {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        LEVEL_CHUNK_WITH_LIGHT.to_id(version)
    }
}

impl<'a> CChunkData<'a> {
    #[must_use]
    pub const fn new(
        chunk_x: i32,
        chunk_z: i32,
        heightmaps: ChunkHeightmaps,
        data: &'a [u8],
        block_entities: Vec<ChunkBlockEntity>,
        light_data: LightData,
    ) -> Self {
        Self {
            chunk_x,
            chunk_z,
            heightmaps,
            data,
            block_entities,
            light_data,
        }
    }
}

impl ClientPacket for CChunkData<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i32_be(self.chunk_x)?;
        write.write_i32_be(self.chunk_z)?;

        self.heightmaps.write_to(&mut write, version)?;

        write.write_var_int(&VarInt(self.data.len() as i32))?;
        write.write_slice(self.data)?;

        write.write_var_int(&VarInt(self.block_entities.len() as i32))?;
        for be in &self.block_entities {
            write.write_u8(be.packed_xz)?;
            write.write_i16_be(be.y)?;
            write.write_var_int(&be.type_id)?;
            write.write_compound_nbt_with_version(Some(&be.data), version)?;
        }

        self.light_data.write(&mut write, version)?;

        Ok(())
    }
}
