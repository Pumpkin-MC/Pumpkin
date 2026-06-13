use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::block_state_remap::remap_block_state_for_version;
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::get_local_cord;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk::{ChunkData, palette::NetworkPalette};
use std::io::Write;

pub struct ChunkBlockEntityData {
    pub position: pumpkin_util::math::position::BlockPos,
    pub type_id: i32,
    pub nbt: pumpkin_nbt::compound::NbtCompound,
}

/// Sent by the server to provide the client with the full data for a chunk.
///
/// This includes heightmaps, the actual block and biome data (organized into sections),
/// block entities (like signs or chests), and the light level information for both
/// sky and block light.
#[java_packet(PLAY_LEVEL_CHUNK_WITH_LIGHT)]
pub struct CChunkData<'a> {
    chunk: &'a ChunkData,
    live_block_entities: &'a [ChunkBlockEntityData],
}

pub(crate) fn write_chunk_light_data(
    chunk: &ChunkData,
    mut write: impl Write,
) -> Result<(), WritingError> {
    // Light masks include sections from -1 (below world) to num_sections (above world).
    let light_engine = chunk
        .light_engine
        .lock()
        .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;
    let num_sections = light_engine.sky_light.len();

    let mut sky_light_empty_mask = 1u64;
    let mut block_light_empty_mask = 1u64;
    let mut sky_light_mask = 0u64;
    let mut block_light_mask = 0u64;

    for section_index in 0..num_sections {
        let bit_index = section_index + 1;

        if let LightContainer::Full(_) = &light_engine.sky_light[section_index] {
            sky_light_mask |= 1 << bit_index;
        } else {
            sky_light_empty_mask |= 1 << bit_index;
        }

        if let LightContainer::Full(_) = &light_engine.block_light[section_index] {
            block_light_mask |= 1 << bit_index;
        } else {
            block_light_empty_mask |= 1 << bit_index;
        }
    }

    sky_light_empty_mask |= 1 << (num_sections + 1);
    block_light_empty_mask |= 1 << (num_sections + 1);

    write.write_bitset(&BitSet(Box::new([sky_light_mask as i64])))?;
    write.write_bitset(&BitSet(Box::new([block_light_mask as i64])))?;
    write.write_bitset(&BitSet(Box::new([sky_light_empty_mask as i64])))?;
    write.write_bitset(&BitSet(Box::new([block_light_empty_mask as i64])))?;

    let light_data_size = VarInt(LightContainer::ARRAY_SIZE as i32);

    write.write_var_int(&VarInt(sky_light_mask.count_ones() as i32))?;
    for section in &light_engine.sky_light {
        if let LightContainer::Full(data) = section {
            write.write_var_int(&light_data_size)?;
            write.write_slice(data)?;
        }
    }

    write.write_var_int(&VarInt(block_light_mask.count_ones() as i32))?;
    for section in &light_engine.block_light {
        if let LightContainer::Full(data) = section {
            write.write_var_int(&light_data_size)?;
            write.write_slice(data)?;
        }
    }

    Ok(())
}

impl<'a> CChunkData<'a> {
    #[must_use]
    pub const fn new(chunk: &'a ChunkData) -> Self {
        Self {
            chunk,
            live_block_entities: &[],
        }
    }

    #[must_use]
    pub const fn with_block_entities(
        chunk: &'a ChunkData,
        live_block_entities: &'a [ChunkBlockEntityData],
    ) -> Self {
        Self {
            chunk,
            live_block_entities,
        }
    }
}

impl ClientPacket for CChunkData<'_> {
    #[expect(clippy::too_many_lines)]
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        // Chunk X
        write.write_i32_be(self.chunk.x)?;
        // Chunk Z
        write.write_i32_be(self.chunk.z)?;

        let heightmaps = self
            .chunk
            .heightmap
            .lock()
            .map_err(|_| WritingError::Message("heightmap lock poisoned".into()))?;
        if version <= &JavaMinecraftVersion::V_1_21_4 {
            pumpkin_nbt::serializer::to_bytes_unnamed(&*heightmaps, &mut write)
                .map_err(|err| WritingError::Serde(err.to_string()))?;
        } else {
            write.write_var_int(&VarInt(3))?; // Map size

            let mut write_heightmap = |index: i32, data: &[i64]| -> Result<(), WritingError> {
                write.write_var_int(&VarInt(index))?;
                write.write_var_int(&VarInt(data.len() as i32))?;
                for val in data {
                    write.write_i64_be(*val)?;
                }
                Ok(())
            };

            write_heightmap(1, heightmaps.world_surface.as_deref().unwrap_or(&[0; 37]))?;
            write_heightmap(4, heightmaps.motion_blocking.as_deref().unwrap_or(&[0; 37]))?;
            write_heightmap(
                5,
                heightmaps
                    .motion_blocking_no_leaves
                    .as_deref()
                    .unwrap_or(&[0; 37]),
            )?;
        }
        drop(heightmaps);

        {
            let mut blocks_and_biomes_buf = Vec::new();
            let block_sections =
                self.chunk.section.block_sections.read().map_err(|_| {
                    WritingError::Message("block_sections read lock poisoned".into())
                })?;
            let biome_sections =
                self.chunk.section.biome_sections.read().map_err(|_| {
                    WritingError::Message("biome_sections read lock poisoned".into())
                })?;

            for (block_palette, biome_palette) in block_sections.iter().zip(biome_sections.iter()) {
                let non_empty_block_count = block_palette.non_air_block_count() as i16;
                blocks_and_biomes_buf.write_i16_be(non_empty_block_count)?;
                if version >= &JavaMinecraftVersion::V_26_1 {
                    // New in 26.1, fluid count
                    let liquid_count = block_palette.liquid_block_count() as i16;
                    blocks_and_biomes_buf.write_i16_be(liquid_count)?;
                }

                let mut block_network = block_palette.convert_network();
                if version < &CURRENT_MC_VERSION {
                    match &mut block_network.palette {
                        NetworkPalette::Single(registry_id) => {
                            *registry_id = remap_block_state_for_version(*registry_id, *version);
                        }
                        NetworkPalette::Indirect(palette) => {
                            for registry_id in palette.iter_mut() {
                                *registry_id =
                                    remap_block_state_for_version(*registry_id, *version);
                            }
                        }
                        NetworkPalette::Direct => {
                            let bits_per_entry = usize::from(block_network.bits_per_entry);
                            let values_per_i64 = 64 / bits_per_entry;
                            let id_mask = (1u64 << bits_per_entry) - 1;

                            for packed_word in &mut block_network.packed_data {
                                let mut remapped_word = 0u64;
                                let packed_word_u64 = *packed_word as u64;
                                for index in 0..values_per_i64 {
                                    let shift = index * bits_per_entry;
                                    let state_id = ((packed_word_u64 >> shift) & id_mask) as u16;
                                    let remapped_id =
                                        remap_block_state_for_version(state_id, *version);
                                    remapped_word |= u64::from(remapped_id) << shift;
                                }
                                *packed_word = remapped_word as i64;
                            }
                        }
                    }
                }
                blocks_and_biomes_buf.write_u8(block_network.bits_per_entry)?;

                match block_network.palette {
                    NetworkPalette::Single(registry_id) => {
                        blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                    }
                    NetworkPalette::Indirect(palette) => {
                        blocks_and_biomes_buf.write_var_int(&palette.len().try_into().map_err(
                            |_| {
                                WritingError::Message(format!(
                                    "{} is not representable as a VarInt!",
                                    palette.len()
                                ))
                            },
                        )?)?;
                        for registry_id in palette {
                            blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                        }
                    }
                    NetworkPalette::Direct => {}
                }

                if version <= &JavaMinecraftVersion::V_1_21_4 {
                    blocks_and_biomes_buf
                        .write_list(&block_network.packed_data, |buf, &packed| {
                            buf.write_i64_be(packed)
                        })?;
                } else {
                    for packed in block_network.packed_data {
                        blocks_and_biomes_buf.write_i64_be(packed)?;
                    }
                }

                let biome_network = biome_palette.convert_network();
                blocks_and_biomes_buf.write_u8(biome_network.bits_per_entry)?;

                match biome_network.palette {
                    NetworkPalette::Single(registry_id) => {
                        blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                    }
                    NetworkPalette::Indirect(palette) => {
                        blocks_and_biomes_buf.write_var_int(&palette.len().try_into().map_err(
                            |_| {
                                WritingError::Message(format!(
                                    "{} is not representable as a VarInt!",
                                    palette.len()
                                ))
                            },
                        )?)?;
                        for registry_id in palette {
                            blocks_and_biomes_buf.write_var_int(&registry_id.into())?;
                        }
                    }
                    NetworkPalette::Direct => {}
                }

                if version <= &JavaMinecraftVersion::V_1_21_4 {
                    blocks_and_biomes_buf
                        .write_list(&biome_network.packed_data, |buf, &packed| {
                            buf.write_i64_be(packed)
                        })?;
                } else {
                    for packed in biome_network.packed_data {
                        blocks_and_biomes_buf.write_i64_be(packed)?;
                    }
                }
            }

            write.write_var_int(&blocks_and_biomes_buf.len().try_into().map_err(|_| {
                WritingError::Message(format!(
                    "{} is not representable as a VarInt!",
                    blocks_and_biomes_buf.len()
                ))
            })?)?;
            write.write_slice(&blocks_and_biomes_buf)?;
        };

        let block_entities = self
            .chunk
            .pending_block_entities
            .lock()
            .map_err(|_| WritingError::Message("block_entities lock poisoned".into()))?;
        let persisted_count = block_entities
            .keys()
            .filter(|pos| {
                !self
                    .live_block_entities
                    .iter()
                    .any(|entity| entity.position == **pos)
            })
            .count();
        write.write_var_int(&VarInt(
            (persisted_count + self.live_block_entities.len()) as i32,
        ))?;
        for (pos, nbt) in block_entities.iter() {
            if self
                .live_block_entities
                .iter()
                .any(|entity| entity.position == *pos)
            {
                continue;
            }
            let local_xz = ((get_local_cord(pos.0.x) & 0xF) << 4) | (get_local_cord(pos.0.z) & 0xF);

            write.write_u8(local_xz as u8)?;
            write.write_i16_be(pos.0.y as i16)?;

            let id = nbt.get_string("id").map_or(0, |id_str| {
                let name = id_str.split(':').next_back().unwrap_or(id_str);
                pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
                    .iter()
                    .position(|&n| n == name)
                    .unwrap_or(0)
            });

            write.write_var_int(&VarInt(id as i32))?;
            write.write_nbt(nbt.clone().into())?;
        }
        drop(block_entities);

        for entity in self.live_block_entities {
            let pos = entity.position;
            let local_xz = ((get_local_cord(pos.0.x) & 0xF) << 4) | (get_local_cord(pos.0.z) & 0xF);

            write.write_u8(local_xz as u8)?;
            write.write_i16_be(pos.0.y as i16)?;
            write.write_var_int(&VarInt(entity.type_id))?;
            write.write_nbt(entity.nbt.clone().into())?;
        }

        write_chunk_light_data(self.chunk, &mut write)?;
        Ok(())
    }
}
