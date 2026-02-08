use crate::WritingError;
use crate::codec::bit_set::BitSet;
use crate::{ClientPacket, VarInt, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_LEVEL_CHUNK_WITH_LIGHT;
use pumpkin_macros::java_packet;
use pumpkin_nbt::END_ID;
use pumpkin_util::math::position::get_local_cord;
use pumpkin_util::version::MinecraftVersion;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk::{ChunkData, palette::NetworkPalette};
use std::io::Write;

/// Sent by the server to provide the client with the full data for a chunk.
///
/// This includes heightmaps, the actual block and biome data (organized into sections),
/// block entities (like signs or chests), and the light level information for both
/// sky and block light.
#[java_packet(PLAY_LEVEL_CHUNK_WITH_LIGHT)]
pub struct CChunkData<'a>(pub &'a ChunkData);

impl ClientPacket for CChunkData<'_> {
    #[expect(clippy::too_many_lines)]
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // Chunk X
        write.write_i32_be(self.0.x)?;
        // Chunk Z
        write.write_i32_be(self.0.z)?;

        let heightmaps = &self.0.heightmap.lock().unwrap();
        write.write_var_int(&VarInt(3))?; // Map size

        let mut write_heightmap = |index: i32, data: &[i64]| -> Result<(), WritingError> {
            write.write_var_int(&VarInt(index))?;
            write.write_var_int(&VarInt(data.len() as i32))?;
            for val in data {
                write.write_i64_be(*val)?;
            }
            Ok(())
        };

        write_heightmap(1, &heightmaps.world_surface)?;
        write_heightmap(4, &heightmaps.motion_blocking)?;
        write_heightmap(5, &heightmaps.motion_blocking_no_leaves)?;

        {
            let mut blocks_and_biomes_buf = Vec::new();
            let block_sections = self.0.section.block_sections.read().unwrap();
            let biome_sections = self.0.section.biome_sections.read().unwrap();

            for (block_palette, biome_palette) in block_sections.iter().zip(biome_sections.iter()) {
                let non_empty_block_count = block_palette.non_air_block_count() as i16;
                blocks_and_biomes_buf.write_i16_be(non_empty_block_count)?;

                let block_network = block_palette.convert_network();
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

                for packed in block_network.packed_data {
                    blocks_and_biomes_buf.write_i64_be(packed)?;
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

                for packed in biome_network.packed_data {
                    blocks_and_biomes_buf.write_i64_be(packed)?;
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

        let block_entities = self.0.block_entities.lock().unwrap();
        write.write_var_int(&VarInt(block_entities.len() as i32))?;
        for block_entity in block_entities.values() {
            let pos = block_entity.get_position();
            let local_xz = ((get_local_cord(pos.0.x) & 0xF) << 4) | (get_local_cord(pos.0.z) & 0xF);

            write.write_u8(local_xz as u8)?;
            write.write_i16_be(pos.0.y as i16)?;
            write.write_var_int(&VarInt(block_entity.get_id() as i32))?;

            if let Some(nbt) = block_entity.chunk_data_nbt() {
                write.write_nbt(nbt.into())?;
            } else {
                write.write_u8(END_ID)?;
            }
        }

        {
            // Light masks include sections from -1 (below world) to num_sections (above world)
            // This means we need to account for 2 extra sections in the bitset
            let light_engine = self.0.light_engine.lock().unwrap();
            let num_sections = light_engine.sky_light.len();

            let mut sky_light_empty_mask = 0u64;
            let mut block_light_empty_mask = 0u64;
            let mut sky_light_mask = 0u64;
            let mut block_light_mask = 0u64;

            // Bit 0 represents the section below the world (always empty)
            sky_light_empty_mask |= 1 << 0;
            block_light_empty_mask |= 1 << 0;

            // Bits 1..=num_sections represent the actual world sections
            for section_index in 0..num_sections {
                let bit_index = section_index + 1; // Offset by 1 for the below-world section

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

            // Bit num_sections+1 represents the section above the world (always empty)
            sky_light_empty_mask |= 1 << (num_sections + 1);
            block_light_empty_mask |= 1 << (num_sections + 1);

            // Write Sky Light Mask
            write.write_bitset(&BitSet(Box::new([sky_light_mask.try_into().unwrap()])))?;
            // Write Block Light Mask
            write.write_bitset(&BitSet(Box::new([block_light_mask.try_into().unwrap()])))?;
            // Write Empty Sky Light Mask
            write.write_bitset(&BitSet(Box::new([sky_light_empty_mask
                .try_into()
                .unwrap()])))?;
            // Write Empty Block Light Mask
            write.write_bitset(&BitSet(Box::new([block_light_empty_mask
                .try_into()
                .unwrap()])))?;

            let light_data_size: VarInt = LightContainer::ARRAY_SIZE.try_into().unwrap();

            // Write Sky Light arrays
            write.write_var_int(&VarInt(sky_light_mask.count_ones() as i32))?;
            for section_index in 0..num_sections {
                if let LightContainer::Full(data) = &light_engine.sky_light[section_index] {
                    write.write_var_int(&light_data_size)?;
                    write.write_slice(data.as_ref())?;
                }
            }

            // Write Block Light arrays
            write.write_var_int(&VarInt(block_light_mask.count_ones() as i32))?;
            for section_index in 0..num_sections {
                if let LightContainer::Full(data) = &light_engine.block_light[section_index] {
                    write.write_var_int(&light_data_size)?;
                    write.write_slice(data.as_ref())?;
                }
            }
        }
        Ok(())
    }
}
